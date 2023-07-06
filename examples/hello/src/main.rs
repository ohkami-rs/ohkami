mod health_handler {
    use ohkami::{Context, Response};

    pub async fn health_check(c: Context) -> Response {
        c.NoContent()
    }
}


mod hello_handler {
    use ohkami::{Context, Response, Query, Payload};

    #[Query]
    pub struct HelloQuery {
        name:   String,
        repeat: Option<usize>,
    }

    pub async fn hello_by_query(c: Context,
        HelloQuery { name, repeat }: HelloQuery
    ) -> Response<String> {
        let message = name.repeat(repeat.unwrap_or(1));
        c.Text(message)
    }


    #[Payload(JSON)]
    #[derive(serde::Deserialize)]
    pub struct HelloRequest {
        name:   String,
        repeat: Option<usize>,
    }

    pub async fn hello_by_json(c: Context,
        HelloRequest { name, repeat }: HelloRequest
    ) -> Response<String> {
        if name.is_empty() {
            return Response::Err(c
                .BadRequest()
                .Text("`name` mustn't be empty")
            )
        }
        
        let message = name.repeat(repeat.unwrap_or(1));
        c.Text(message)
    }
}


mod fang {
    use ohkami::{Context, Request};

    pub async fn append_server(c: &mut Context) {
        tracing::info!("\
            Called `append_server`\n\
        ");

        c.headers
            .Server("ohkami");
    }

    pub async fn log_request(req: &Request) {
        let __method__ = req.method();
        let __path__   = req.path();

        tracing::info!("\
            Got request:\n\
            [ method ] {__method__}\n\
            [  path  ] {__path__}\n\
        ");
    }
}


#[tokio::main]
async fn main() {
    use ohkami::{Ohkami, GlobalFangs, Route};

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    GlobalFangs::new()
        .NotFound(|nf| nf
            .Text("No resource for that request found")
        )
        .apply();

    let hello_ohkami = Ohkami::with((fang::append_server,))(
        "/query".
            GET(hello_handler::hello_by_query),
        "/json".
            POST(hello_handler::hello_by_json),
    );

    tracing::info!("Started listening on http://localhost:3000");

    Ohkami::with((fang::log_request,))(
        "/hc" .GET(health_handler::health_check),
        "/api".by(hello_ohkami),
    ).howl(3000).await
}
