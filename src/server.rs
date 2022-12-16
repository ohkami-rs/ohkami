use async_std::{
    sync::Arc,
    task::block_on,
    io::WriteExt,
    net::{TcpStream, TcpListener},
    stream::StreamExt, task,
};
use tracing_subscriber::fmt::SubscriberBuilder;
use std::{collections::HashMap, pin::Pin, future::Future};
use crate::{
    components::{
        method::Method, cors::CORS
    },
    context::Context,
    response::Response,
    result::{Result, ElseResponse},
    utils::{
        parse::parse_request_lines, validation::{self, is_valid_path}, buffer::Buffer
    },
};

#[cfg(feature = "postgres")]
use sqlx::postgres::{
    PgPool as ConnectionPool,
    PgPoolOptions as PoolOptions,
};
#[cfg(feature = "mysql")]
use sqlx::mysql::{
    MySqlPool as ConnectionPool,
    MySqlPoolOptions as PoolOptions,
};

type Handler = Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>;

pub struct Server {
    map: HashMap<
        (Method, &'static str, /* /*with param tailing or not*/bool */),
        Handler,
    >,
    cors: CORS,

    #[cfg(feature = "sqlx")]
    pool: ConnectionPool,
}

pub struct Config<#[cfg(feature = "sqlx")] 'url> {
    pub cors: CORS,
    pub log_subscribe: Option<SubscriberBuilder>,

    #[cfg(feature = "sqlx")]
    pub db_profile: DBprofile<'url>,
}
#[cfg(not(feature = "sqlx"))]
impl Default for Config {
    fn default() -> Self {
        Self {
            cors:          CORS::default(),
            log_subscribe: Some(tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG)),
        }
    }
}
#[cfg(feature = "sqlx")]
impl<'url> Default for Config<'url> {
    fn default() -> Self {
        Self {
            cors:          CORS::default(),
            log_subscribe: Some(tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG)),
            db_profile:    DBprofile::default(),
        }
    }
}

#[cfg(feature = "sqlx")]
pub struct DBprofile<'url> {
    pub pool_options: PoolOptions,
    pub url:          &'url str,
}
#[cfg(feature = "sqlx")]
impl<'url> Default for DBprofile<'url> {
    fn default() -> Self {
        Self {
            pool_options: PoolOptions::default(),
            url:          "empty url",
        }
    }
}

impl Server {
    #[cfg(not(feature = "sqlx"))]
    pub fn setup() -> Self {
        let default_config = Config::default();

        if let Some(subscriber) = default_config.log_subscribe {
            subscriber.init()
        }

        Self {
            map:  HashMap::new(),
            cors: default_config.cors,
        }
    }
    pub fn setup_with(config: Config) -> Self {
        if let Some(subscriber) = config.log_subscribe {
            subscriber.init()
        }

        #[cfg(feature = "sqlx")]
        let pool = {
            let DBprofile { pool_options, url } = config.db_profile;
            let err_msg = format!("Can't connect to DB at {url} with {pool_options:?}. If you won't deal with any database, you shouldn't enable `sqlx` flag");

            let pool_connection = block_on(pool_options.connect(url));
            if pool_connection.is_err() {
                tracing::error!(err_msg);
                panic!()
            }

            pool_connection.unwrap()
        };

        Self {
            map:  HashMap::new(),
            cors: config.cors,

            #[cfg(feature = "sqlx")]
            pool
        }
    }

    #[allow(non_snake_case)]
    pub fn GET<'ctx, Fut: Future<Output = Result<Response>> + Send + 'static>(self,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        self.add_handler(Method::GET, path_string, handler)
    }
    #[allow(non_snake_case)]
    pub fn POST<'ctx, Fut: Future<Output = Result<Response>> + Send + 'static>(self,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        self.add_handler(Method::POST, path_string, handler)
    }
    #[allow(non_snake_case)]
    pub fn PATCH<'ctx, Fut: Future<Output = Result<Response>> + Send + 'static>(self,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        self.add_handler(Method::PATCH, path_string, handler)
    }
    #[allow(non_snake_case)]
    pub fn DELETE<'ctx, Fut: Future<Output = Result<Response>> + Send + 'static>(self,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        self.add_handler(Method::DELETE, path_string, handler)
    }

    fn add_handler<'ctx, Fut: Future<Output = Result<Response>> + Send + 'static>(mut self,
        method:      Method,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        if !is_valid_path(path_string) {
            panic!("`{path_string}` is invalid as path.");
        }

        let path = // (path, has_param) =
            if let Some((path, _param_name)) = path_string.rsplit_once("/:") {
                path // (path, true)
            } else {
                path_string // (path_string, false)
            };

        if self.map.insert(
            (method, &path.trim_end_matches('/'), /*has_param*/),
            Box::new(move |ctx: Context| Box::pin(handler(ctx)))
        ).is_some() {
            panic!("handler for `{method} {path_string}` is resistered duplicatedly");
        }

        self
    }

    pub fn serve_on(self, address: &'static str) -> Result<()> {
        tracing::info!("started seving on {}...", address);
        let tcp_address = validation::tcp_address(address);

        let allow_origin_str = Arc::new(
            if self.cors.allow_origins.is_empty() {
                String::new()
            } else {
                format!("Access-Control-Allow-Origin: {}", self.cors.allow_origins.join(" "))
            }
        );

        let handler_map = Arc::new(self.map);

        #[cfg(feature = "sqlx")]
        let connection_pool = Arc::new(self.pool);

        block_on(async {
            let listener = TcpListener::bind(tcp_address).await?;
            while let Some(stream) = listener.incoming().next().await {
                let stream = stream?;
                task::spawn(
                    handle_stream(
                        stream,
                        Arc::clone(&handler_map),
                        Arc::clone(&allow_origin_str),
                        
                        #[cfg(feature = "sqlx")]
                        Arc::clone(&connection_pool),
                    )
                );
            }
            Ok(())
        })
    }
}

async fn handle_stream(
    mut stream: TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, /*bool*/),
        Handler,
    >>,
    allow_origin_str: Arc<String>,

    #[cfg(feature = "sqlx")]
    connection_pool:  Arc<ConnectionPool>,
) {
    let mut response = match setup_response(
        &mut stream,
        handler_map,

        #[cfg(feature = "sqlx")]
        connection_pool,
    ).await {
        Ok(res)  => res,
        Err(res) => res,
    };

    if !allow_origin_str.is_empty() {
        response.add_header(&*allow_origin_str)
    }

    tracing::debug!("generated a response: {:?}", &response);

    if let Err(err) = response.write_to_stream(&mut stream).await {
        tracing::error!("failed to write response: {}", err);
        return
    }
    if let Err(err) = stream.flush().await {
        tracing::error!("failed to flush stream: {}", err);
        return
    }
}

async fn setup_response(
    stream: &mut TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, /*bool*/),
        Handler
    >>,

    #[cfg(feature = "sqlx")]
    connection_pool: Arc<ConnectionPool>,
) -> Result<Response> {
    let buffer = Buffer::new(stream).await?;

    let (
        method,
        path,
        mut param_range,
        query_range,
        // headers,
        body
    ) = parse_request_lines(
        buffer.lines()?
    )?;

    let handler = {
        match handler_map.get(&(
            method,
            &path,
            /*false*/
        )) {
            Some(handler) => {
                param_range = None; //
                handler
            },
            None => handler_map.get(&(
                method,
                &path.rsplit_once('/').unwrap_or(("", "")).0,
                // true
            ))._else(|| Response::NotFound(format!(
                "handler for `{method} {path}` is not found"
            )))?
        }
    };

    let context = Context {
        buffer,
        body,
        param_range,
        query_range,

        #[cfg(feature = "sqlx")]
        connection_pool
    };

    handler(context).await
}
