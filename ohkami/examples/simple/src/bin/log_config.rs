use ohkami::prelude::*;

fn main() -> Result<()> {
    let config = Config {
        log_subscribe:
            Some(tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
            ),
        ..Default::default()
    };

    Ohkami::with(config)
        .GET("/", || async {Response::OK("Hello!")})
        .howl(":5000")
}