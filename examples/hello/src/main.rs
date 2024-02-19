mod health_handler {
    use ohkami::typed::status::NoContent;

    pub async fn health_check() -> NoContent {
        NoContent
    }
}


mod hello_handler {
    use ohkami::{Response, Status};
    use ohkami::typed::{Payload, Query};

    #[Query]
    pub struct HelloQuery<'q> {
        name:   &'q str,
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


    #[Payload(JSOND)]
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
    use ohkami::{Request, Response, FrontFang, BackFang};

    pub struct SetServer;
    impl BackFang for SetServer {
        type Error = std::convert::Infallible;
        async fn bite(&self, res: &mut Response, _: &Request) -> Result<(), Self::Error> {
            res.headers.set()
                .Server("ohkami");

            tracing::info!("\
                Called `SetServer`\n\
                [current headers]\n\
                {:?}\n\
            ", res.headers);

            Ok(())
        }
    }

    pub struct LogRequest;
    impl FrontFang for LogRequest {
        type Error = std::convert::Infallible;
        async fn bite(&self, req: &mut Request) -> Result<(), Self::Error> {
            let __method__ = req.method();
            let __path__   = req.path();

            tracing::info!("\n\
                Got request:\n\
                [ method ] {__method__}\n\
                [  path  ] {__path__}\n\
            ");

            Ok(())
        }
    }
}


#[tokio::main]
async fn main() {
    use ohkami::prelude::*;
    use fangs::*;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let hello_ohkami = Ohkami::with(SetServer, (
        "/query".
            GET(hello_handler::hello_by_query),
        "/json".
            POST(hello_handler::hello_by_json),
    ));

    tracing::info!("Started listening on http://localhost:3000");

    Ohkami::new((
        "/hc" .GET(health_handler::health_check),
        "/api".By(hello_ohkami),
    )).howl_with(LogRequest, "localhost:3000").await
}
