use ohkami::prelude::*;

fn main() -> Result<()> {
    let config = Config {
        log_subscribe:
            Some(tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
            ),
        ..Default::default()
    };

    Server::setup_with(config)
        .GET("/", |_| async {Response::OK(Body::text("Hello!"))})
        .serve_on(":5000")
}