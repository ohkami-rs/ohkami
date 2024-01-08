mod config;
mod errors;
mod models;
mod repositories;
mod fangs;
mod handlers;

use errors::RealWorldError;


#[tokio::main]
async fn main() -> Result<(), RealWorldError> {
    config::init().await?;

    handlers::realworld_ohkami()
        .howl(":8080").await;

    Ok(())
}
