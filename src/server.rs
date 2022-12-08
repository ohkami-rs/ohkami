use async_std::{
    sync::Arc,
    task::block_on,
    io::{ReadExt, WriteExt},
    net::{TcpStream, TcpListener},
    stream::StreamExt, task,
};
use std::{collections::HashMap, pin::Pin};
use futures::Future;
use crate::{
    components::{
        consts::BUF_SIZE, method::Method, cors::CORS
    },
    context::Context,
    response::Response,
    result::Result,
    utils::{
        parse::parse_stream, validation::{self, is_valid_path}
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

#[cfg(not(feature = "sqlx"))]
type Handler = Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>;
#[cfg(feature = "sqlx")]
type Handler = Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>;

pub struct Server {
    map: HashMap<
        (Method, &'static str, /*with param tailing or not*/bool),
        Handler,
    >,
    cors: CORS,

    #[cfg(feature = "sqlx")]
    pool: ConnectionPool,
}

pub struct Config<'url> {
    pub cors: CORS,

    #[cfg(feature = "sqlx")]
    pub connection_pool_of: (PoolOptions, &'url str),//Option<ConnectionPool>,
}
impl<'url> Default for Config<'url> {
    fn default() -> Self {
        Self {
            cors:               CORS::default(),
            connection_pool_of: (PoolOptions::new(), "empty URL"),
        }
    }
}

impl Server {
    #[cfg(not(feature = "sqlx"))]
    pub fn setup() -> Self {
        Self {
            map:    HashMap::new(),
            cors:   CORS::default(),
        }
    }
    pub fn setup_with(config: Config) -> Self {
        let (pool_options, url) = config.connection_pool_of;
        let err_msg = format!("Can't connect to DB at {url} with {pool_options:?}. If you won't deal with any database, you shouldn't enable `sqlx` flag");

        Self {
            map:  HashMap::new(),
            cors: config.cors,

            #[cfg(feature = "sqlx")]
            pool: block_on(pool_options.connect(url)).expect(&err_msg)
        }
    }

    #[allow(non_snake_case)]
    pub fn GET<Fut: Future<Output = Result<Response>> + Send + 'static>(self,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        self.add_handler(Method::GET, path_string, handler)
    }
    #[allow(non_snake_case)]
    pub fn POST<Fut: Future<Output = Result<Response>> + Send + 'static>(self,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        self.add_handler(Method::POST, path_string, handler)
    }
    #[allow(non_snake_case)]
    pub fn PATCH<Fut: Future<Output = Result<Response>> + Send + 'static>(self,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        self.add_handler(Method::PATCH, path_string, handler)
    }
    #[allow(non_snake_case)]
    pub fn DELETE<Fut: Future<Output = Result<Response>> + Send + 'static>(self,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        self.add_handler(Method::DELETE, path_string, handler)
    }

    fn add_handler<#[cfg(feature = "sqlx")] 'ctx,
        Fut: Future<Output = Result<Response>> + Send + 'static,
    >(mut self,
        method:      Method,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        if !is_valid_path(path_string) {
            panic!("`{path_string}` is invalid as path.");
        }

        let (path, has_param) =
            if let Some((path, _param_name)) = path_string.rsplit_once("/:") {
                (path, true)
            } else {
                (path_string, false)
            };

        if self.map.insert(
            (method, &path, has_param), Box::new(move |ctx| Box::pin(handler(ctx)))
        ).is_some() {
            panic!("handler for `{method} {path_string}` is resistered duplicatedly");
        }

        self
    }

    // #[tracing::instrument(
    //     name = "server setting",
    //     skip(self)
    // )]
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

#[cfg(not(feature = "sqlx"))]
// #[tracing::instrument(
//     name = "server",
//     skip(handler_map)
// )]
async fn handle_stream(
    mut stream: TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        Handler,
    >>,
    allow_origin_str: Arc<String>,
) {
    let mut response = match setup_response(
        &mut stream,
        handler_map,
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
#[cfg(feature = "sqlx")]
// #[tracing::instrument(
//     name = "server",
//     skip(handler_map)
// )]
async fn handle_stream(
    mut stream: TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        Handler,
    >>,
    allow_origin_str: Arc<String>,
    connection_pool:  Arc<ConnectionPool>,
) {
    let mut response = match setup_response(
        &mut stream,
        handler_map,
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

#[cfg(not(feature = "sqlx"))]
async fn setup_response(
    stream: &mut TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        Handler
    >>,
) -> Result<Response> {
    let mut buffer = [b' '; BUF_SIZE];
    stream.read(&mut buffer).await?;
    let (
        method,
        path_str,
        context
    ) = parse_stream(
        &buffer
    )?;

    handle_request(
        handler_map,
        method,
        path_str,
        context
    ).await
}
#[cfg(feature = "sqlx")]
async fn setup_response(
    stream: &mut TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        Handler
    >>,
    connection_pool: Arc<ConnectionPool>,
) -> Result<Response> {
    let mut buffer = [b' '; BUF_SIZE];
    stream.read(&mut buffer).await?;
    let (
        method,
        path_str,
        context
    ) = parse_stream(
        &buffer,
        connection_pool
    )?;

    handle_request(
        handler_map,
        method,
        path_str,
        context
    ).await
}

async fn handle_request<'req>(
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        Handler
    >>,
    method:   Method,
    path_str: &'req str,
    mut request_context: Context,
) -> Result<Response> {
    let (path, param) = {
        let (rest, tail) = path_str.rsplit_once('/').unwrap();
        if let Ok(param) = tail.parse::<u32>() {
            request_context.param = Some(param);
            (rest, Some(param))
        } else {
            (path_str, None)
        }
    };

    let handler = handler_map
        .get(&(method, path, param.is_some()))
        .ok_or_else(|| Response::NotFound(format!("handler for `{method} {path_str}` is not found")))?;

    handler(request_context).await
}
