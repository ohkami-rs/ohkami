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

#[cfg(feature = "potgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;

// #[derive(Debug)]
pub struct Server {
    map: HashMap<
        (Method, &'static str, /*with param tailing or not*/bool),
        // fn(Context) -> Result<Response>,
        // Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>
        Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>
    >,
    cors: CORS,

    #[cfg(feature = "sqlx")]
    pool:   Option<ConnectionPool>,
}
// pub struct ServerSetting {
//     map: HashMap<
//         (Method, &'static str, bool),
//         // Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>,
//         Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>
//     >,
//     cors:   CORS,
//     errors: Vec<String>,
// 
//     #[cfg(feature = "sqlx")]
//     pool:   Option<ConnectionPool>,
// }
pub struct Config {
    pub cors: CORS,

    #[cfg(feature = "sqlx")]
    pub db_connection_pool: Option<ConnectionPool>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            cors: CORS::default(),

            #[cfg(feature = "sqlx")]
            db_connection_pool: None,
        }
    }
}

impl Server { // ServerSetting {
    pub fn setup() -> Self {
        Self {
            map:    HashMap::new(),
            cors:   CORS::default(),
            // errors: Vec::new(),
   
            #[cfg(feature = "sqlx")]
            pool:   None,
        }
    }
    pub fn setup_with(config: Config) -> Self {
        Self {
            map:    HashMap::new(),
            cors:   config.cors,
            // errors: Vec::new(),

            #[cfg(feature = "sqlx")]
            pool:   config.db_connection_pool,
        }
    }

    // #[tracing::instrument(
    //     name = "server setting",
    //     skip(self)
    // )]
    pub fn serve_on(self, address: &'static str) -> Result<()> {
        // if !self.errors.is_empty() {
        //     tracing::error!("got a SetupError:");
        //     for error in &self.errors {
        //         tracing::error!("{}", error)
        //     }
        // }

        // let server = Server {
        //     map:  self.map.clone(),
        //     cors: self.cors.clone(),
// 
        //     #[cfg(feature = "sqlx")]
        //     pool: self.pool,
        // };

        tracing::info!("started seving on {}...", address);
        let tcp_address = validation::tcp_address(address);

        let allow_origin_str = Arc::new(
            if self.cors.allow_origins.is_empty() {
                String::new()
            } else {
                format!("Access-Control-Allow-Origin: {}", self.cors.allow_origins.join(" "))
            }
        );
        
        #[cfg(feature = "sqlx")]
        let connection_pool = Arc::new(self.pool);
        
        let handler_map = Arc::new(self.map);

        block_on(
            // server.serve_on(
            //     validation::tcp_address(address)
            // )
            // // .instrument(
            // //     tracing::debug_span!("server")
            // // )
        async {
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
//     async fn serve_on(self, tcp_address: String) -> Result<()> {
//         let handler_map = Arc::new(self.map);
//         let allow_origin_str = Arc::new(
//             if self.cors.allow_origins.is_empty() {
//                 String::new()
//             } else {
//                 format!("Access-Control-Allow-Origin: {}", self.cors.allow_origins.join(" "))
//             }
//         );
// 
//         #[cfg(feature = "sqlx")]
//         let connection_pool = Arc::new(self.pool);
// 
//         let listener = TcpListener::bind(tcp_address).await?;
//         let mut incoming = listener.incoming();
// 
//         while let Some(stream) = incoming.next().await {
//             let stream = stream?;
//             task::spawn(
//                 handle_stream(
//                     stream,
//                     Arc::clone(&handler_map),
//                     Arc::clone(&handler_map),
// 
//                     #[cfg(feature = "sqlx")]
//                     Arc::clone(&connection_pool),
//                 )
//                 // .instrument(
//                 //     tracing::debug_span!("handle stream")
//                 // )
//             );
//         }
// 
//         Ok(())
//     }

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

    fn add_handler<Fut: Future<Output = Result<Response>> + Send + 'static>(mut self,
        method:      Method,
        path_string: &'static str,
        handler:     fn(Context) -> Fut,
    ) -> Self {
        if !is_valid_path(path_string) {
            // self.errors.push(format!("`{path_string}` is invalid as path."));
            // return self
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
            // self.errors.push(format!("handler for `{method} {path_string}` is resistered duplicatedly"))
            panic!("handler for `{method} {path_string}` is resistered duplicatedly");
        }

        self
    }
}
// impl Server {
//     pub fn setup() -> ServerSetting {
//         ServerSetting {
//             map:    HashMap::new(),
//             cors:   CORS::default(),
//             errors: Vec::new(),
// 
//             #[cfg(feature = "sqlx")]
//             pool:   None,
//         }
//     }
//     pub fn setup_with(config: Config) -> ServerSetting {
//         ServerSetting {
//             map:    HashMap::new(),
//             cors:   config.cors,
//             errors: Vec::new(),
// 
//             #[cfg(feature = "sqlx")]
//             pool:   config.db_connection_pool,
//         }
//     }
// 
//     #[tracing::instrument(
//         name = "server", skip(self)
//     )]
//     async fn serve_on(self, tcp_address: String) -> Result<()> {
//         let handler_map = Arc::new(self.map);
//         let allow_origin_str = Arc::new(
//             if self.cors.allow_origins.is_empty() {
//                 String::new()
//             } else {
//                 format!("Access-Control-Allow-Origin: {}", self.cors.allow_origins.join(" "))
//             }
//         );
// 
//         #[cfg(feature = "sqlx")]
//         let connection_pool = Arc::new(self.pool);
// 
//         let listener = TcpListener::bind(tcp_address).await?;
//         let mut incoming = listener.incoming();
// 
//         while let Some(stream) = incoming.next().await {
//             let stream = stream?;
//             task::spawn(
//                 handle_stream(
//                     stream,
//                     Arc::clone(&handler_map),
//                     Arc::clone(&allow_origin_str),
// 
//                     #[cfg(feature = "sqlx")]
//                     Arc::clone(&connection_pool),
//                 )
//                 // .instrument(
//                 //     tracing::debug_span!("handle stream")
//                 // )
//             );
//         }
// 
//         Ok(())
//     }
// }

#[cfg(not(feature = "sqlx"))]
#[tracing::instrument(
    name = "server",
    skip(handler_map)
)]
async fn handle_stream(
    mut stream: TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>,
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
//     name = "server"
// )]
async fn handle_stream(
    mut stream: TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        Box<dyn Handler>,
    >>,
    allow_origin_str: Arc<String>,
    connection_pool:  Arc<Option<ConnectionPool>>,
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
        Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>
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
        Box<dyn Handler>
    >>,
    connection_pool: Arc<Option<ConnectionPool>>,
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

    context.pool = connection_pool.as_ref().as_ref();

    handle_request(
        handler_map,
        method,
        path_str,
        context
    ).await
}

async fn handle_request<'ctx>(
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output=Result<Response>> + Send >> + Send + Sync>
    >>,
    method:   Method,
    path_str: &'ctx str,
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
