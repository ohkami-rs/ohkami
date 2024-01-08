use std::sync::OnceLock;
use serde::Deserialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use crate::errors::RealWorldError;


#[derive(Deserialize)]
pub struct JWTPayload {
    pub iat:     u64,
    pub user_id: String,
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
    static DB_URL: OnceLock<String> = OnceLock::new();
    let db_url = DB_URL.get_or_init(|| match std::env::var("DB_URL") {
        Ok(url) => url,
        Err(e)  => panic!("Required envirinment variable `DB_URL` is not found: {e}"),
    });

    static JWT_SECRET_KEY: OnceLock<String> = OnceLock::new();
    let jwt_secret_key = JWT_SECRET_KEY.get_or_init(|| match std::env::var("JWT_SECRET_KEY") {
        Ok(key) => key,
        Err(e)  => panic!("Required envirinment variable `JWT_SECRET_KEY` is not found: {e}"),
    });


    PG_POOL.set(
        PgPoolOptions::new()
            .connect(db_url).await
            .map_err(|e| RealWorldError::DB(e))?
    ).map_err(|_| RealWorldError::Config("`PG_POOL` is already set unexpectedly"))?;

    OUR_JWT.set(
        ohkami::utils::JWT(jwt_secret_key)
    ).map_err(|_| RealWorldError::Config("`OUR_JWT` is already set unexpectedly"))?;


    Ok(())
}
