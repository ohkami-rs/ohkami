use ohkami::{prelude::*, components::cors::CORS};

fn main() -> Result<()> {
    Server::setup_with(Config {
        cors: CORS {
            allow_origins: &["http://localhost:8000"],
            ..Default::default()
        }
    })
        .GET("/", hello)
        .serve_on(":5000")
}

fn hello(_: Context) -> Result<Response> {
    Response::OK(
        JSON::from("Hello!")
    )
}