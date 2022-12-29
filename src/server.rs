use async_std::{
    sync::Arc,
    task::block_on,
    io::WriteExt,
    net::{TcpStream, TcpListener},
    stream::StreamExt, task,
};
use tracing_subscriber::fmt::SubscriberBuilder;
use crate::{
    components::method::Method,
    context::{Context, RequestContext},
    response::Response,
    result::Result,
    utils::{
        parse::parse_request_lines, validation, buffer::Buffer
    },
    router::Router,
    handler::{Handler, Param, group::HandlerGroup}, setting::{IntoServerSetting, ServerSetting, Middleware}
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
pub struct Ohkami {
    pub(crate) router: Router,
    log_subscribe: Option<SubscriberBuilder>,

    #[cfg(feature = "sqlx")]
    pub(crate) pool: Arc<ConnectionPool>,

    pub(crate) setup_errors: Vec<String>,
    middleware_register: Middleware,
}


#[cfg(feature = "sqlx")]
pub struct DBprofile<'url> {
    pub options: PoolOptions,
    pub url:     &'url str,
}
#[cfg(feature = "sqlx")]
impl<'url> Default for DBprofile<'url> {
    fn default() -> Self {
        Self {
            options: PoolOptions::default(),
            url:     "empty url",
        }
    }
}

#[cfg(not(feature = "sqlx"))]
impl Default for Ohkami {
    fn default() -> Self {
        let ServerSetting {
            config,
            middleware
        } = ServerSetting::default();

        Self {
            router: Router::new(),
            log_subscribe: config.log_subscribe,
            setup_errors: Vec::new(),
            middleware_register: middleware,
        }
    }
}

impl Ohkami {
    /// Initialize `Ohkami` with given configuratoin.
    #[cfg(not(feature = "sqlx"))]
    pub fn with<ISS: IntoServerSetting>(setting: ISS) -> Self {
        let ServerSetting { 
            config,
            middleware,
        } = setting.into_setting();

        Self {
            router: Router::new(),
            log_subscribe: config.log_subscribe,
            setup_errors: Vec::new(),
            middleware_register: middleware,
        }
    }
    #[cfg(feature = "sqlx")]
    pub fn with<'db_url, ISS: IntoServerSetting<'db_url>>(setting: ISS) -> Self {
        let ServerSetting { 
            config,
            middleware,
        } = setting.into_setting();

        let pool = {
            let DBprofile { options, url } = config.db_profile;
            let err_msg = format!("Can't connect to DB at {url} with {options:?}. If you won't deal with any database, you shouldn't enable `sqlx` flag");

            let pool_connection = block_on(options.connect(url));
            if pool_connection.is_err() {
                tracing::error!(err_msg);
                panic!()
            }

            pool_connection.unwrap()
        };

        Self {
            router: Router::new(),
            log_subscribe: config.log_subscribe,

            #[cfg(feature = "sqlx")]
            pool: Arc::new(pool),

            setup_errors: Vec::new(),
            middleware_register: middleware,
        }
    }

    /// Add a handler to request `GET /*path*/ HTTP/1.1`. valid path format:
    /// 
    /// `/ | (/:?[a-z, A-Z, _ ]+)+`
    /// 
    /// Sections starting with `:` are a path parameters.
    /// 
    /// ```no_run
    /// Ohkami::setup()
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
    /// Ohkami::setup()
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
        ) = validation::valid_request_path(path);
        if !is_valid_path {
            self.setup_errors.push(format!("`{path}` is invalid as path."));
        }

        let (
            handler,
            expect_param_num
        ) = handler.into_handlefunc();
        if param_count < expect_param_num {
            self.setup_errors.push(format!("handler for `{method} {path}` expects {expect_param_num} path params, this is more than actual ones: ({param_count})"))
        }

        if let Err(msg) = self.router.register(
            method,
            &path.trim_end_matches('/'),
            handler
        ) {
            self.setup_errors.push(format!("{msg}"))
        }

        self
    }

    pub fn route(mut self, path: &'static str, group: HandlerGroup) -> Self {
        let (
            is_valid_path,
            param_count
        ) = validation::valid_request_path(path);
        if !is_valid_path {
            self.setup_errors.push(format!("`{path}` is invalid as path."));
        }

        let HandlerGroup {
            max_param_count,
            GET,
            POST,
            PATCH,
            DELETE,
        } = group;

        if param_count < max_param_count {
            self.setup_errors.push(format!("handlers for `{path}` expect {max_param_count} path params, this is more than actual ones: ({param_count})"))
        }

        if let Some(handler) = GET  {
            if let Err(msg) = self.router.register(
                Method::GET,
                &path.trim_end_matches('/'),
                handler
            ) {
                self.setup_errors.push(format!("{msg}"))
            }
        }
        if let Some(handler) = POST  {
            if let Err(msg) = self.router.register(
                Method::POST,
                &path.trim_end_matches('/'),
                handler
            ) {
                self.setup_errors.push(format!("{msg}"))
            }
        }
        if let Some(handler) = PATCH  {
            if let Err(msg) = self.router.register(
                Method::PATCH,
                &path.trim_end_matches('/'),
                handler
            ) {
                self.setup_errors.push(format!("{msg}"))
            }
        }
        if let Some(handler) = DELETE  {
            if let Err(msg) = self.router.register(
                Method::DELETE,
                &path.trim_end_matches('/'),
                handler
            ) {
                self.setup_errors.push(format!("{msg}"))
            }
        }

        self
    }

    /// Start listening and serving on given TCP address (if it failed, returns error).\
    /// - `":{port}"` (like `":3000"`) is interpret as `"0.0.0.0:{port}"`
    /// - `"localhost:{port}"` (like `"localhost:8080"`) is interpret as `"127.0.0.1:{port}"`
    /// - other formats are interpret as raw TCP address
    pub fn howl(mut self, address: &'static str) -> Result<()> {
        if let Some(subscriber) = self.log_subscribe {
            subscriber.init();
        }

        let router = Arc::new({
            let applied_router = self.router.apply(self.middleware_register);
            if let Err(msg) = &applied_router {
                self.setup_errors.push(msg.to_owned())
            }
            applied_router.unwrap()
        });

        if ! self.setup_errors.is_empty() {
            for err in self.setup_errors {
                tracing::error!(err)
            }
            return Err(Response::InternalServerError("can't serve"))
        }
        drop(self.setup_errors);

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
                        Arc::clone(&router),
                        
                        #[cfg(feature = "sqlx")]
                        Arc::clone(&self.pool),
                    )
                );
            }
            Ok(())
        })
    }
}


async fn handle_stream(
    mut stream: TcpStream,
    router: Arc<Router>,

    #[cfg(feature = "sqlx")]
    connection_pool:  Arc<ConnectionPool>,
) {
    let response = match setup_response(
        &mut stream,
        router,

        #[cfg(feature = "sqlx")]
        connection_pool,
    ).await {
        Ok(res)  => res,
        Err(res) => res,
    };

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
    router: Arc<Router>,

    #[cfg(feature = "sqlx")]
    connection_pool: Arc<ConnectionPool>,
) -> Result<Response> {
    let buffer = Buffer::new(stream).await?;
    consume_buffer(
        buffer,
        &*router,
        
        #[cfg(feature = "sqlx")]
        connection_pool.clone(),
    ).await
}

pub(crate) async fn consume_buffer(
    buffer: Buffer,
    router: &Router,

    #[cfg(feature = "sqlx")]
    connection_pool: Arc<ConnectionPool>,
) -> Result<Response> {
    let (
        method,
        path,
        query_range,
        headers,
        body
    ) = parse_request_lines(
        buffer.lines()?
    )?;

    let (
        handler,
        params,
        middleware_proccess,
        middleware_just,
    ) = router.search(
        method,
        &path
    )?;

    let mut context = Context {
        req: RequestContext {
            buffer,
            // body,
            headers,
            query_range,
        },
        additional_headers: String::new(),
        
        #[cfg(feature = "sqlx")]
        pool: connection_pool,
    };

    for proccess in middleware_proccess {
        context = proccess(context).await;
    }
    if let Some(pre_handle) = middleware_just {
        context = pre_handle(context).await;
    }

    tracing::debug!("{:?}", context);

    handler(context, params, body).await
}


#[cfg(test)]
mod test {
    use crate::{prelude::*, setting::Config};

    #[test]
    fn basic_use() {
        let config = Config {
            log_subscribe: None,
            ..Default::default()
        };

        Ohkami::with(config)
            .GET("/", || async {
                Response::OK("Hello!")
            });
    }
}