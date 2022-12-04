use ohkami::{prelude::*, json};
use once_cell::sync::Lazy;
use serde::Deserialize;
use sqlx::FromRow;

static DB_URL: Lazy<String> = Lazy::new(|| format!(
    "postgres://{}:{}@{}:{}/{}",
    std::env::var("POSTGRES_USER").unwrap(),
    std::env::var("POSTGRES_PASSWORD").unwrap(),
    std::env::var("POSTGRES_HOST").unwrap(),
    std::env::var("POSTGRES_PORT").unwrap(),
    std::env::var("POSTGRES_DB").unwrap(),
));

fn main() -> Result<()> {
    let pool = useDB(async {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .connect(&DB_URL)
            .await
    })?;

    Server::setup_with(Config {
        db_connection_pool: Some(pool),
        ..Default::default()
    })
        .POST("/login", post_login)
        .serve_on(":3000")
}

#[derive(FromRow)]
struct User {
    id:       i64,
    email:    String,
    password: String,
}
#[derive(Deserialize)]
struct LoginRequest {
    email:    String,
    password: String,
}

fn post_login(ctx: Context) -> Result<Response> {
    let request_body = ctx.request_body::<LoginRequest>()
        .else_response(|res| res.error_context("Can't get email and password"))?;
    (request_body.email.len() > 0 && request_body.password.len() > 0)
        .else_response(|| Response::BadRequest("Empty email or password"))?;
    
    let user = useDB(async {
        sqlx::query_as::<_, User>("SELECT * from users WHERE email = $1")
            .bind(request_body.email)
            .fetch_one(ctx.pool())
            .await
    })?;

    // Hash the password in `request body` and check if it equals to the password in `user`.

    let token = "sample_new_token_for_this_user";
    Response::OK(
        json!{"token": token}
    )
}