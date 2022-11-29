use std::collections::HashMap;
use async_std::{
    sync::Arc,
    task::block_on,
    io::{ReadExt, WriteExt},
    net::{TcpStream, TcpListener},
    stream::StreamExt, task,
};
use sqlx::PgPool;
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
    pool: Option<PgPool>,
    cors: CORS,
}
pub struct ServerSetting {
    map: HashMap<
        (Method, &'static str, bool),
        fn(Context) -> Result<Response>,
    >,
    pool:   Option<PgPool>,
    cors:   CORS,
    errors: Vec<String>,
}


impl ServerSetting {
    pub fn serve_on(&self, address: &'static str) -> Result<()> {
        if !self.errors.is_empty() {
            return Response::SetUpError(&self.errors)
        }

        let server = Server {
            map:  self.map.clone(),
            pool: self.pool.clone(),
            cors: self.cors.clone(),
        };

        let tcp_address = validation::tcp_address(address);
        block_on(
            server.serve_on(tcp_address)
        )
    }
    
    pub fn cors(&mut self, cors: CORS) -> &mut Self {
        self.cors = cors;
        self
    }
    pub fn db_connection_pool(&mut self, pool: PgPool) -> &mut Self {
        self.pool = Some(pool);
        self
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
            pool:   None,
            cors:   CORS::default(),
            errors: Vec::new(),
        }
    }

    async fn serve_on(self, tcp_address: String) -> Result<()> {
        let handler_map = Arc::new(self.map);
        let connection_pool = Arc::new(self.pool);
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
                    Arc::clone(&connection_pool),
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
    connection_pool: Arc<Option<PgPool>>,
    allow_origin_str: Arc<String>,
) {
    let mut response = match setup_response(
        &mut stream,
        connection_pool,
        handler_map,
    ).await {
        Ok(res)  => res,
        Err(res) => res,
    };

    response.add_header(&*allow_origin_str);

    if let Err(err) = response.write_to_stream(&mut stream).await {
        eprintln!("failed to write response: {}", err)
    }
    if let Err(err) = stream.flush().await {
        eprintln!("failed to flush stream: {}", err)   
    }
}

async fn setup_response(
    stream: &mut TcpStream,
    connection_pool: Arc<Option<PgPool>>,
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
        mut context
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
        fn(Context) -> Result<Response>,
    >>,
    method:   Method,
    path_str: &'ctx str,
    mut request_context: Context<'ctx>,
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

    handler(request_context)
}
