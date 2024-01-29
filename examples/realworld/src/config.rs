use ohkami::utils::{Deserialize, Serialize};
use std::sync::OnceLock;
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
pub fn pepper() -> &'static [u8] {
    PEPPER.get().expect("`config::init` hasn't been called")
}

static PG_POOL: OnceLock<PgPool>             = OnceLock::new();
static OUR_JWT: OnceLock<ohkami::utils::JWT> = OnceLock::new();
static PEPPER:  OnceLock<Vec<u8>>            = OnceLock::new();


pub async fn init() -> Result<(), RealWorldError> {
    dotenvy::dotenv().map_err(|e| RealWorldError::Config(format!("Failed to load .env: {e}")))?;

    let db_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(e)  => panic!("Required envirinment variable `DATABASE_URL` is not found: {e}"),
    };

    let jwt_secret_key = match std::env::var("JWT_SECRET_KEY") {
        Ok(key) => key,
        Err(e)  => panic!("Required envirinment variable `JWT_SECRET_KEY` is not found: {e}"),
    };

    let pepper = match std::env::var("PEPPER") {
        Ok(p)   => p,
        Err(e)  => panic!("Required envirinment variable `PEPPER` is not found: {e}"),
    };

    PG_POOL.set(
        PgPoolOptions::new()
            .max_connections(42)
            .min_connections(42)
            .connect(&db_url).await
            .map_err(|e| RealWorldError::DB(e))?
    ).map_err(|_| RealWorldError::Config("`PG_POOL` is already set unexpectedly".to_string()))?;

    OUR_JWT.set(
        ohkami::utils::JWT(jwt_secret_key)
    ).map_err(|_| RealWorldError::Config("`OUR_JWT` is already set unexpectedly".to_string()))?;

    PEPPER.set(
        pepper.into_bytes()
    ).map_err(|_| RealWorldError::Config("`PEPPER` is already set unexpectedly".to_string()))?;

    Ok(())
}
