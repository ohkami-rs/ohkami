#[cfg(not(any(feature = "postgres", feature = "mysql")))] pub(crate) mod server {

use async_std::{
    sync::Arc,
    task::block_on,
    io::{ReadExt, WriteExt},
    net::{TcpStream, TcpListener},
    stream::StreamExt, task,
};
use std::collections::HashMap;
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

pub struct Server {
    map: HashMap<
        (Method, &'static str, bool),
        fn(Context) -> Result<Response>,
    >,
    cors: CORS,
}
pub struct ServerSetting {
    map: HashMap<
        (Method, &'static str, bool),
        fn(Context) -> Result<Response>,
    >,
    cors:   CORS,
    errors: Vec<String>,
}
pub struct Config {
    pub cors: CORS,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            cors: CORS::default(),
        }
    }
}

impl ServerSetting {
    pub fn serve_on(&self, address: &'static str) -> Result<()> {
        if !self.errors.is_empty() {
            return Err(Response::SetUpError(&self.errors))
        }

        let server = Server {
            map:  self.map.clone(),
            cors: self.cors.clone(),
        };

        let tcp_address = validation::tcp_address(address);
        block_on(
            server.serve_on(tcp_address)
        )
    }

    #[allow(non_snake_case)]
    pub fn GET(&mut self,
        path_string: &'static str,
        handler:     fn(Context) -> Result<Response>,
    ) -> &mut Self {
        self.add_handler(Method::GET, path_string, handler)
    }
    #[allow(non_snake_case)]
    pub fn POST(&mut self,
        path_string: &'static str,
        handler:     fn(Context) -> Result<Response>,
    ) -> &mut Self {
        self.add_handler(Method::POST, path_string, handler)
    }
    #[allow(non_snake_case)]
    pub fn PATCH(&mut self,
        path_string: &'static str,
        handler:     fn(Context) -> Result<Response>,
    ) -> &mut Self {
        self.add_handler(Method::PATCH, path_string, handler)
    }
    #[allow(non_snake_case)]
    pub fn DELETE(&mut self,
        path_string: &'static str,
        handler:     fn(Context) -> Result<Response>,
    ) -> &mut Self {
        self.add_handler(Method::DELETE, path_string, handler)
    }

    fn add_handler(&mut self,
        method:      Method,
        path_string: &'static str,
        handler:     fn(Context) -> Result<Response>,
    ) -> &mut Self {
        if !is_valid_path(path_string) {
            self.errors.push(format!("`{path_string}` is invalid as path."));
            return self
        }

        let (path, has_param) =
            if let Some((path, _param_name)) = path_string.rsplit_once("/:") {
                (path, true)
            } else {
                (path_string, false)
            };

        
        if self.map.insert(
            (method, &path, has_param), handler
        ).is_some() {
            self.errors.push(format!("handler for `{method} {path_string}` is resistered duplicatedly"))
        }

        self
    }
}
impl Server {
    pub fn setup() -> ServerSetting {
        ServerSetting {
            map:    HashMap::new(),
            cors:   CORS::default(),
            errors: Vec::new(),
        }
    }
    pub fn setup_with(config: Config) -> ServerSetting {
        ServerSetting {
            map:    HashMap::new(),
            cors:   config.cors,
            errors: Vec::new()
        }
    }

    async fn serve_on(self, tcp_address: String) -> Result<()> {
        let handler_map = Arc::new(self.map);
        let allow_origin_str = Arc::new(
            if self.cors.allow_origins.is_empty() {
                String::new()
            } else {
                format!("Access-Control-Allow-Origin: {}", self.cors.allow_origins.join(" "))
            }
        );

        let listener = TcpListener::bind(tcp_address).await?;
        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            task::spawn(
                handle_stream(
                    stream,
                    Arc::clone(&handler_map),
                    Arc::clone(&allow_origin_str),
                )
            );
        }

        Ok(())
    }
}


async fn handle_stream(
    mut stream: TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        fn(Context) -> Result<Response>,
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

    if let Err(err) = response.write_to_stream(&mut stream).await {
        eprintln!("failed to write response: {}", err)
    }
    if let Err(err) = stream.flush().await {
        eprintln!("failed to flush stream: {}", err)   
    }
}

async fn setup_response(
    stream: &mut TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        fn(Context) -> Result<Response>,
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

async fn handle_request<'ctx>(
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        fn(Context) -> Result<Response>,
    >>,
    method:   Method,
    path_str: &'ctx str,
    mut request_context: Context,
) -> Result<Response> {
    let (path, param) = {
        let (rest, tail) = path_str.rsplit_once('/').unwrap();
        if let Ok(param) = tail.parse::<u32>() {
            request_context.path_param = Some(param);
            (rest, Some(param))
        } else {
            (path_str, None)
        }
    };

    let handler = handler_map
        .get(&(method, path, param.is_some()))
        .ok_or_else(|| Response::NotFound(format!("handler for `{method} {path_str}` is not found")))?;

    handler(request_context)
}

}