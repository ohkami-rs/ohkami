use std::sync::OnceLock;
use ohkami::util::unix_timestamp;
use ohkami::serde::{Serialize, Deserialize};
use ohkami::fang::{Jwt, JwtToken};
use uuid::Uuid;
use crate::errors::RealWorldError;


macro_rules! environment_variables {
    ( $( $name:ident ),* $(,)? ) => {
        $(
            #[allow(non_snake_case)]
            pub fn $name() -> Result<&'static str, RealWorldError> {
                static $name: OnceLock<Result<String, std::env::VarError>> = OnceLock::new();
            
                match $name.get_or_init(|| std::env::var(stringify!($name))) {
                    Ok(value) => Ok(&**value),
                    Err(e) => Err(RealWorldError::Config(e.to_string())),
                }
            }
        )*
    };
} environment_variables! {
    DATABASE_URL,
    PEPPER,
    JWT_SECRET_KEY,
}

#[derive(Serialize, Deserialize)]
pub struct JwtPayload {
    pub iat:     u64,
    pub user_id: Uuid,
}

pub fn issue_jwt_for_user_of_id(user_id: Uuid) -> Result<JwtToken, RealWorldError> {
    let secret = JWT_SECRET_KEY()?;
    Ok(Jwt::default(secret).clone().issue(JwtPayload {
        user_id,
        iat: unix_timestamp(),
    }))
}
