use std::sync::OnceLock;
use serde::Deserialize;


#[allow(non_snake_case)]
pub fn DB_URL() -> &'static str {
    static DB_URL: OnceLock<String> = OnceLock::new();

    DB_URL.get_or_init(|| std::env::var("DB_URL")
        .expect("Required environment variable `DB_URL` is not found"))
}

#[allow(non_snake_case)]
pub fn JWT_SECRET_KEY() -> &'static str {
    static JWT_SECRET_KEY: OnceLock<String> = OnceLock::new();

    JWT_SECRET_KEY.get_or_init(|| 
        std::env::var("JWT_SECRET_KEY")
            .expect("Required envirinment variable `JWT_SECRET_KEY` is not found")
    )
}

#[derive(Deserialize)]
pub struct JWTPayload {
    pub iat:     u64,
    pub user_id: String,
}


