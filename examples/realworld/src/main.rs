mod config;
mod errors;
mod models;
mod repositories;
mod fangs;
mod handler;

use errors::RealWorldError;


#[tokio::main]
async fn main() -> Result<(), RealWorldError> {
    config::init().await?;

    handler::realworld_ohkami()
        .howl(":8080").await;

    Ok(())
}
