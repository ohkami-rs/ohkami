mod models;
mod handlers;

use handlers::{root, user::create_user};
use ohkami::{
    result::Result,
    server::Server, prelude::Config,
};

fn main() -> Result<()> {
    let config = Config {
        log_subscribe: Some(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
        ),
        ..Default::default()
    };

    Server::setup_with(config)
        .GET("/",       root)
        .POST("/users", create_user)
        .serve_on(":3000")
}
