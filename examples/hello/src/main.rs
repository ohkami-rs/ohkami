mod health_handler {
    use ohkami::typed::status::NoContent;

    pub async fn health_check() -> NoContent {
        NoContent
    }
}


mod hello_handler {
    use ohkami::{Response, Status};
    use ohkami::typed::{Payload, Query};
    use ohkami::builtin::payload::JSON;

    #[Query]
    pub struct HelloQuery<'q> {
        name:   &'q str,
        #[query(rename = "n")]
        repeat: Option<usize>,
    }

    pub async fn hello_by_query<'h>(
        HelloQuery { name, repeat }: HelloQuery<'h>
    ) -> String {
        tracing::info!("\
            Called `hello_by_query`\
        ");

        name.repeat(repeat.unwrap_or(1))
    }


    #[Payload(JSON/D)]
    pub struct HelloRequest<'n> {
        name:   &'n str,
        repeat: Option<usize>,
    }

    pub enum ValidationError {
        NameIsEmpty
    }
    impl ohkami::IntoResponse for ValidationError {
        fn into_response(self) -> Response {
            match self {
                Self::NameIsEmpty => Response::with(Status::BadRequest).text("`name` mustn't be empty")
            }
        }
    }

    pub async fn hello_by_json<'h>(
        HelloRequest { name, repeat }: HelloRequest<'h>
    ) -> Result<String, ValidationError> {
        tracing::info!("\
            Called `hello_by_json`\
        ");
        
        if name.is_empty() {
            return Err(ValidationError::NameIsEmpty)
        }
        
        Ok(name.repeat(repeat.unwrap_or(1)))
    }
}


mod fangs {
    use ohkami::prelude::*;

    #[derive(Clone)]
    pub struct SetServer;
    impl FangAction for SetServer {
        fn back<'a>(&'a self, res: &'a mut Response) -> impl std::future::Future<Output = ()> + Send {
            res.headers.set()
                .Server("ohkami");

            tracing::info!("\
                Called `SetServer`\n\
                [current headers]\n\
                {:?}\n\
            ", res.headers);

            async {}
        }
    }

    #[derive(Clone)]
    pub struct LogRequest;
    impl FangAction for LogRequest {
        fn fore<'a>(&'a self, req: &'a mut Request) -> impl std::future::Future<Output = Result<(), Response>> + Send {
            let __method__ = req.method();
            let __path__   = req.path();

            tracing::info!("\n\
                Got request:\n\
                [ method ] {__method__}\n\
                [  path  ] {__path__}\n\
            ");

            async {Ok(())}
        }
    }
}


#[tokio::main]
async fn main() {
    use ohkami::prelude::*;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let hello_ohkami = Ohkami::with((
        fangs::SetServer,
    ), (
        "/query".
            GET(hello_handler::hello_by_query),
        "/json".
            POST(hello_handler::hello_by_json),
    ));

    tracing::info!("Started listening on http://localhost:3000");

    Ohkami::with((
        fangs::LogRequest,
    ), (
        "/hc" .GET(health_handler::health_check),
        "/api".By(hello_ohkami),
    )).howl("localhost:3000").await
}
