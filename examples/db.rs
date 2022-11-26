use cobalt::prelude::*;
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::FromRow;

static DB_URL: Lazy<String> = Lazy::new(|| format!(
    "postgres://{}:{}@{}:{}/{}",
    std::env::var("POSTGRES_HOST").unwrap(),
    std::env::var("POSTGRES_PORT").unwrap(),
    std::env::var("POSTGRES_USER").unwrap(),
    std::env::var("POSTGRES_PASSWORD").unwrap(),
    std::env::var("POSTGRES_DB").unwrap(),
));

fn main() -> Result<()> {
    let pool = useDB(async {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(&DB_URL)
            .await
    })?;
    Server::setup()
        .db_connection_pool(pool)
        .GET("/users/:id", get_user_user_id)
        .serve_on(":3000")
}

#[derive(FromRow, Serialize)]
struct User {
    id:   i64,
    name: String,
}

fn get_user_user_id(ctx: Context) -> Result<Response> {
    let user_id = ctx.param
        .else_response(|| Response::BadRequest("Expected user id as path parameter"))?;

    let user = useDB(async {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id as i64)
        .fetch_one(ctx.pool())
        .await
    })?;

    Response::Created(
        JSON::from_struct(&user)?
    )
}

