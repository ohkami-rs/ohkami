mod health_handler {
    use ohkami::typed::status::NoContent;

    pub async fn health_check() -> NoContent {
        NoContent
    }
}


mod hello_handler {
    use ohkami::format::{Query, JSON};
    use ohkami::serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct HelloQuery<'q> {        
        #[serde(rename = "n")]
        repeat: Option<usize>,
        name: &'q str,
    }

    pub async fn hello_by_query<'h>(
        Query(HelloQuery { name, repeat }): Query<HelloQuery<'h>>
    ) -> String {
        tracing::info!("\
            Called `hello_by_query`\
        ");

        name.repeat(repeat.unwrap_or(1))
    }


    #[derive(Deserialize)]
    pub struct HelloRequest<'n> {
        name:   &'n str,
        repeat: Option<usize>,
    }
    impl HelloRequest<'_> {
        fn validate(&self) -> Result<(), &'static str> {
            let _: () = (! self.name.is_empty()).then_some(())
                .ok_or_else(|| "`name` mustn't be empty")?;

            let _: () = (self.repeat.unwrap_or_default() < 10).then_some(())
                .ok_or_else(|| "`repeat` must be less than 10")?;

            Ok(())
        }
    }

    pub async fn hello_by_json(
        JSON(HelloRequest { name, repeat }): JSON<HelloRequest<'_>>
    ) -> String {
        tracing::info!("\
            Called `hello_by_json`\
        ");
        
        name.repeat(repeat.unwrap_or(1))
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
            let __method__ = req.method;
            let __path__   = req.path.str();

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
