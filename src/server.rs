use async_std::{
    sync::Arc,
    task::block_on,
    io::WriteExt,
    net::{TcpStream, TcpListener},
    stream::StreamExt, task,
};
use tracing_subscriber::fmt::SubscriberBuilder;
use crate::{
    components::{
        method::Method, cors::CORS, headers::Header
    },
    context::Context,
    response::Response,
    result::Result,
    utils::{
        parse::parse_request_lines, validation, buffer::Buffer
    },
    router::Router,
    handler::{Handler, Param}, middleware::Middleware,
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


/// Type of ohkami's server instance
pub struct Server {
    pub(crate) map:  Router<'static>,
    cors: CORS,

    #[cfg(feature = "sqlx")]
    pub(crate) pool: Arc<ConnectionPool>,
}
/// Configurations of `Server`. In current version, this holds
/// 
/// - `cors: CORS`,
/// - `log_subscribe: Option<SubscriberBuilder>`,
/// - `db_profile: DBprofile<'url>` (if feature = "sqlx")
/// 
/// Here, `log_subscribe`'s default value is
/// ```no_run
/// Some(tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG))
/// ```
/// When you'd like to customize this, add `tracing` and `tracing_subscriber` in your dependencies to write custom config like
/// ```no_run
/// fn main() -> Result<()> {
///     let config = Config {
///         log_subscribe: Some(
///             tracing_subscriber::fmt()
///                 .with_max_level(tracing::Level::TRACE)
///         ),
///         ..Default::default()
///     };
/// }
/// ```
pub struct Config<#[cfg(feature = "sqlx")] 'url> {
    pub cors: CORS,
    pub log_subscribe: Option<SubscriberBuilder>,

    middleware: Middleware,

    #[cfg(feature = "sqlx")]
    pub db_profile: DBprofile<'url>,
}
#[cfg(not(feature = "sqlx"))]
impl Default for Config {
    fn default() -> Self {
        Self {
            cors:          CORS::default(),
            log_subscribe: Some(tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG)),
            middleware:    Middleware::init(),
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
            middleware:    Middleware::init(),
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
    /// Just a shortcut of `setup_with(Config::default())`
    #[cfg(not(feature = "sqlx"))]
    pub fn setup() -> Self {
        let default_config = Config::default();

        if let Some(subscriber) = default_config.log_subscribe {
            subscriber.init()
        }

        Self {
            map:  Router::new(),
            cors: default_config.cors,
        }
    }
    /// Initialize `Server` with given configuratoin. This **automatically performe `subscriber.init()`** if config's `log_subscribe` is `Some`, so **DON'T write it in your `main` function**.
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
            map:  Router::new(),
            cors: config.cors,

            #[cfg(feature = "sqlx")]
            pool: Arc::new(pool)
        }
    }

    /// Add a handler to request `GET /*path*/ HTTP/1.1`. valid path format:
    /// 
    /// `/ | (/:?[a-z, A-Z, _ ]+)+`
    /// 
    /// Sections starting with `:` are a path parameters.
    /// 
    /// ```no_run
    /// Server::setup()
    ///     .GET("/api/users/:id", handler)
    /// ```
    #[allow(non_snake_case)]
    pub fn GET<H: Handler<P>, P: Param>(self,
        path:    &'static str,
        handler: H,
    ) -> Self {
        self.add_handler(Method::GET, path, handler)
    }

    /// Add a handler to request `POST /*path*/ HTTP/1.1`. valid path format:
    /// 
    /// `/ | (/:?[a-z, A-Z, _ ]+)+`
    /// 
    /// Sections starting with `:` are a path parameters.
    /// 
    /// ```no_run
    /// Server::setup()
    ///     .POST("/api/users/:id", handler)
    /// ```
    #[allow(non_snake_case)]
    pub fn POST<H: Handler<P>, P: Param>(self,
        path:    &'static str,
        handler: H,
    ) -> Self {
        self.add_handler(Method::POST, path, handler)
    }

    /// Add a handler to request `PATCH /*path*/ HTTP/1.1`. valid path format:
    /// 
    /// `/ | (/:?[a-z, A-Z, _ ]+)+`
    /// 
    /// Sections starting with `:` are a path parameters.
    /// 
    /// ```no_run
    /// Server::setup()
    ///     .PATCH("/api/users/:id", handler)
    /// ```
    #[allow(non_snake_case)]
    pub fn PATCH<H: Handler<P>, P: Param>(self,
        path:    &'static str,
        handler: H,
    ) -> Self {
        self.add_handler(Method::PATCH, path, handler)
    }

    /// Add a handler to request `DELETE /*path*/ HTTP/1.1`. valid path format:
    /// 
    /// `/ | (/:?[a-z, A-Z, _ ]+)+`
    /// 
    /// Sections starting with `:` are a path parameters.
    /// 
    /// ```no_run
    /// Server::setup()
    ///     .DELETE("/api/users/:id", handler)
    /// ```
    #[allow(non_snake_case)]
    pub fn DELETE<H: Handler<P>, P: Param>(self,
        path:    &'static str,
        handler: H,
    ) -> Self {
        self.add_handler(Method::DELETE, path, handler)
    }

    fn add_handler<H: Handler<P>, P: Param>(mut self,
        method:  Method,
        path:    &'static str,
        handler: H,
    ) -> Self {
        let (
            is_valid_path,
            param_count
        ) = validation::valid_path(path);
        if !is_valid_path {
            panic!("`{path}` is invalid as path.");
        }

        let (
            handler,
            expect_param_num
        ) = handler.into_handlefunc();
        if param_count < expect_param_num {
            panic!("handler for `{path}` expects {expect_param_num} path params, this is more than actual ones {param_count}!")
        }

        if let Err(msg) = self.map.register(
            method,
            if path == "/" {"/"} else {&path.trim_end_matches('/')},
            handler
        ) {
            panic!("{msg}")
        }

        self
    }

    /// Start listening and serving on given TCP address (if it failed, returns error).\
    /// - `":{port}"` (like `":3000"`) is interpret as `"0.0.0.0:{port}"`
    /// - `"localhost:{port}"` (like `"localhost:8080"`) is interpret as `"127.0.0.1:{port}"`
    /// - other formats are interpret as raw TCP address
    pub fn serve_on(self, address: &'static str) -> Result<()> {
        let allow_origins_str = Arc::new(
            if self.cors.allow_origins.is_empty() {
                String::new()
            } else {
                self.cors.allow_origins.join(" ")
            }
        );

        let handler_map = Arc::new(self.map);

        block_on(async {
            let listener = TcpListener::bind(
                validation::tcp_address(address)
            ).await?;
            tracing::info!("started seving on {}...", address);
            while let Some(stream) = listener.incoming().next().await {
                let stream = stream?;
                task::spawn(
                    handle_stream(
                        stream,
                        Arc::clone(&handler_map),
                        Arc::clone(&allow_origins_str),
                        
                        #[cfg(feature = "sqlx")]
                        Arc::clone(&self.pool),
                    )
                );
            }
            Ok(())
        })
    }
}

pub trait ExpectedResponse {fn as_response(self) -> Result<Response>;}
impl ExpectedResponse for Response {fn as_response(self) -> Result<Response> {Err(self)}}
impl ExpectedResponse for Result<Response> {fn as_response(self) -> Result<Response> {self}}


async fn handle_stream(
    mut stream: TcpStream,
    handler_map: Arc<Router<'static>>,
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
        response.add_header(Header::AccessControlAllowOrigin, &*allow_origin_str)
    }

    tracing::info!("generated a response: {:?}", &response);

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
    handler_map: Arc<Router<'static>>,

    #[cfg(feature = "sqlx")]
    connection_pool: Arc<ConnectionPool>,
) -> Result<Response> {
    let buffer = Buffer::new(stream).await?;
    consume_buffer(
        buffer,
        &*handler_map,
        
        #[cfg(feature = "sqlx")]
        connection_pool.clone(),
    ).await
}

pub(crate) async fn consume_buffer(
    buffer:          Buffer,
    handler_map:     &Router<'static>,

    #[cfg(feature = "sqlx")]
    connection_pool: Arc<ConnectionPool>,
) -> Result<Response> {
    let (
        method,
        path,
        query_range,
        // headers,
        body
    ) = parse_request_lines(
        buffer.lines()?
    )?;

    let (
        handler,
        params
    ) = handler_map.search(
        method,
        &path
    )?;

    let context = Context {
        buffer,
        body,
        query_range,

        #[cfg(feature = "sqlx")]
        pool: connection_pool,
    };

    tracing::debug!("context: {:#?}", context);

    handler(context, params).await
}


#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn basic_use() {
        let config = Config {
            log_subscribe: None,
            ..Default::default()
        };

        Server::setup_with(config)
            .GET("/", || async {
                Response::OK("Hello!")
            });
    }
}