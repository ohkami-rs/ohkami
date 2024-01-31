mod db;
mod config;
mod errors;
mod models;
mod fangs;
mod handlers;

use errors::RealWorldError;
use sqlx::postgres::PgPoolOptions;


#[tokio::main]
async fn main() -> Result<(), errors::RealWorldError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    dotenvy::dotenv().map_err(|e| RealWorldError::Config(format!("Failed to load .env: {e}")))?;

    let db_url = std::env::var("DB_URL").map_err(|e| RealWorldError::Config(e.to_string()))?;
    let pool = PgPoolOptions::new()
        .max_connections(42)
        .min_connections(42)
        .connect(&db_url).await
        .map_err(|e| RealWorldError::DB(e))?;

    handlers::realworld_ohkami(pool)
        .howl(":8080").await;

    Ok(())
}
