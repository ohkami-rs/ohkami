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
        .GET("/", || async {Response::OK("Hello!")})
        .serve_on(":5000")
}