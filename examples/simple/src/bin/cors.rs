use ohkami::{prelude::*, components::cors::CORS};

fn main() -> Result<()> {
    let config = Config {
        cors: CORS {
            allow_origins: &["http://localhost:8000"],
            ..Default::default()
        },
        ..Config::default()
    };

    Server::setup_with(config)
        .GET("/", hello)
        .serve_on(":5000")
}

async fn hello(_: Context) -> Result<Response> {
    Response::OK(
        JSON::from("Hello!")
    )
}