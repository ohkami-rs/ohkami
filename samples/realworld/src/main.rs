mod db;
mod config;
mod errors;
mod models;
mod fangs;
mod handlers;

#[cfg(test)]
mod _test;

use errors::RealWorldError;

use sqlx::postgres::PgPoolOptions;


#[tokio::main]
async fn main() -> Result<(), errors::RealWorldError> {
    dotenvy::dotenv()
        .map_err(|e| RealWorldError::Config(format!("Failed to load .env: {e}")))?;
    tracing_subscriber::fmt()
        .with_max_level(tracing_subscriber::filter::LevelFilter::DEBUG)
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(42)
        .min_connections(42)
        .connect(config::DATABASE_URL()?).await
        .map_err(|e| RealWorldError::DB(e))?;

    handlers::realworld_ohkami(pool)
        .howl("localhost:8080").await;

    Ok(())
}
