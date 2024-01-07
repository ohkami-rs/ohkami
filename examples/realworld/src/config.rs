use std::sync::OnceLock;
use serde::Deserialize;


#[allow(non_snake_case)]
pub fn JWT_SECRET_KEY() -> &'static str {
    static JWT_SECRET_KEY: OnceLock<String> = OnceLock::new();
    JWT_SECRET_KEY.get_or_init(|| {
        std::env::var("JWT_SECRET_KEY")
            .expect("Envirinment variable `JWT_SECRET_KEY` is not found")
    })
}

#[derive(Deserialize)]
pub struct JWTPayload {
    pub iat:      u64,
    pub username: String,
}
