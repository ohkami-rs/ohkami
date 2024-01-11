use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::{PgPool, postgres::PgPoolOptions};
use crate::errors::RealWorldError;


#[derive(Serialize, Deserialize)]
pub struct JWTPayload {
    pub iat:     u64,
    pub user_id: Uuid,
}

pub fn issue_jwt_for_user_of_id(user_id: Uuid) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    jwt().clone().issue(JWTPayload {
        user_id,
        iat: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    })
}

pub fn pool() -> &'static PgPool {
    PG_POOL.get().expect("`config::init` hasn't been called")
}
pub fn jwt() -> &'static ohkami::utils::JWT {
    OUR_JWT.get().expect("`config::init` hasn't been called")
}

static PG_POOL: OnceLock<PgPool>             = OnceLock::new();
static OUR_JWT: OnceLock<ohkami::utils::JWT> = OnceLock::new();

pub async fn init() -> Result<(), RealWorldError> {
    dotenvy::dotenv().map_err(|e| RealWorldError::Config(format!("Failed to load .env: {e}")))?;

    static DATABASE_URL: OnceLock<String> = OnceLock::new();
    let db_url = DATABASE_URL.get_or_init(|| match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(e)  => panic!("Required envirinment variable `DATABASE_URL` is not found: {e}"),
    });

    static JWT_SECRET_KEY: OnceLock<String> = OnceLock::new();
    let jwt_secret_key = JWT_SECRET_KEY.get_or_init(|| match std::env::var("JWT_SECRET_KEY") {
        Ok(key) => key,
        Err(e)  => panic!("Required envirinment variable `JWT_SECRET_KEY` is not found: {e}"),
    });

    PG_POOL.set(
        PgPoolOptions::new()
            .max_connections(42)
            .min_connections(42)
            .connect(db_url).await
            .map_err(|e| RealWorldError::DB(e))?
    ).map_err(|_| RealWorldError::Config("`PG_POOL` is already set unexpectedly".to_string()))?;

    OUR_JWT.set(
        ohkami::utils::JWT(jwt_secret_key)
    ).map_err(|_| RealWorldError::Config("`OUR_JWT` is already set unexpectedly".to_string()))?;

    Ok(())
}
