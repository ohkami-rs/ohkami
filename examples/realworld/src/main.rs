mod db;
mod config;
mod errors;
mod models;
mod fangs;
mod handlers;


#[tokio::main]
async fn main() -> Result<(), errors::RealWorldError> {
    config::init().await?;
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    handlers::realworld_ohkami()
        .howl(":8080").await;

    Ok(())
}
