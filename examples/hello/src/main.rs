mod health_handler {
    use ohkami::typed::NoContent;

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


    #[Payload(JSON)]
    #[derive(serde::Deserialize)]
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
            Called `hello_by_query`\
        ");
        
        if name.is_empty() {
            return Err(ValidationError::NameIsEmpty)
        }
        
        Ok(name.repeat(repeat.unwrap_or(1)))
    }
}


mod fangs {
    use ohkami::{Request, Fang, IntoFang, Response};

    pub struct SetServer;
    impl IntoFang for SetServer {
        fn into_fang(self) -> Fang {
            Fang::back(|res: &mut Response| {
                res.headers.set()
                    .Server("ohkami");

                tracing::info!("\
                    Called `append_server`\n\
                    [current headers]\n\
                    {:?}\
                ", res.headers);
            })
        }
    }

    pub struct LogRequest;
    impl IntoFang for LogRequest {
        fn into_fang(self) -> Fang {
            Fang::front(|req: &Request| {
                let __method__ = req.method();
                let __path__   = req.path();

                tracing::info!("\
                    Got request:\n\
                    [ method ] {__method__}\n\
                    [  path  ] {__path__}\n\
                ");
            })
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

    let hello_ohkami = Ohkami::with((SetServer, LogRequest), (
        "/query".
            GET(hello_handler::hello_by_query),
        "/json".
            POST(hello_handler::hello_by_json),
    ));

    tracing::info!("Started listening on http://localhost:3000");

    Ohkami::with(LogRequest, (
        "/hc" .GET(health_handler::health_check),
        "/api".By(hello_ohkami),
    )).howl(3000).await
}
