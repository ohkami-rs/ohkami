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
        c.headers
            .Server("ohkami");
    }

    pub async fn log_request(req: &Request) {
        let __method__ = req.method();
        let __path__   = req.path();

        tracing::info!("\
            Gor request:\n\
            [method] {__method__} \n\
            [path]   {__path__} \n\
        ");
    }
}



#[tokio::main]
async fn main() {
    use ohkami::{Ohkami, GlobalFangs, Route};

    GlobalFangs::new()
        .NotFound(|nf| nf
            .Text("Noe resource for that request found")
        )
        .apply();

    let health_ohkami = Ohkami::new()(
        "/".
            GET(health_handler::health_check)
    );

    let hello_ohkami = Ohkami::with((fang::append_server,))(
        "/query".
            GET(hello_handler::hello_by_query),
        "/json".
            POST(hello_handler::hello_by_json),
    );

    Ohkami::with((fang::log_request,))(
        "/hc" .by(health_ohkami),
        "/api".by(hello_ohkami),
    ).howl(3000).await
}
