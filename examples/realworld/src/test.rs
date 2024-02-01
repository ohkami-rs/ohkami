use crate::{config, handlers};
use sqlx::postgres::PgPoolOptions;
use ohkami::testing::*;

#[tokio::test] async fn senario() {
    dotenvy::dotenv().unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(42)
        .min_connections(42)
        .connect(config::DATABASE_URL().unwrap()).await
        .unwrap();

    let t = handlers::realworld_ohkami(pool);

    
}
