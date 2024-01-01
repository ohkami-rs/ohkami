mod health_handler {
    use ohkami::{Context, Response};

    pub async fn health_check(c: Context) -> Response {
        c.NoContent()
    }
}


mod hello_handler {
    use ohkami::{Context, Response};
    use ohkami::utils::{Payload, Query};

    #[Query]
    pub struct HelloQuery<'q> {
        name:   &'q str,
        repeat: Option<usize>,
    }

    pub async fn hello_by_query<'h>(c: Context,
        HelloQuery { name, repeat }: HelloQuery<'h>
    ) -> Response {
        tracing::info!("\
            Called `hello_by_query`\
        ");

        let message = name.repeat(repeat.unwrap_or(1));
        c.OK().text(message)
    }


    #[Payload(JSON)]
    #[derive(serde::Deserialize)]
    pub struct HelloRequest<'n> {
        name:   &'n str,
        repeat: Option<usize>,
    }

    pub async fn hello_by_json<'h>(c: Context,
        HelloRequest { name, repeat }: HelloRequest<'h>
    ) -> Response {
        tracing::info!("\
            Called `hello_by_query`\
        ");
        
        if name.is_empty() {
            return c
                .BadRequest()
                .text("`name` mustn't be empty")
        }
        
        let message = name.repeat(repeat.unwrap_or(1));
        c.OK().text(message)
    }
}


mod fangs {
    use ohkami::{Context, Request, Fang, IntoFang};

    pub struct SetServer;
    impl IntoFang for SetServer {
        fn into_fang(self) -> Fang {
            Fang(|c: &mut Context| {
                c.set_headers()
                    .Server("ohkami");

                tracing::info!("\
                    Called `append_server`\n\
                    [current headers]\n\
                    {:?}\
                ", c.headers);
            })
        }
    }

    pub struct LogRequest;
    impl IntoFang for LogRequest {
        fn into_fang(self) -> Fang {
            Fang(|req: &Request| {
                let __method__ = req.method;
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
