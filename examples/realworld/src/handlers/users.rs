use ohkami::{Ohkami, Route, utils::{Payload, JSON}};
use serde::Deserialize;
use crate::{models::{User, UserResponse}, errors::RealWorldError, config::pool};


pub fn users_ohkami() -> Ohkami {
    Ohkami::new((
        "/login"
            .POST(login),
        "/"
            .POST(register),
    ))
}

#[Payload(JSON)]
struct LoginRequest<'req> {
    user: LoginRequestUser<'req>,
}
impl<'req> Deserialize<'req> for LoginRequest<'req> {
    fn deserialize<D: serde::Deserializer<'req>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self {
            user: LoginRequestUser::deserialize(deserializer)?,
        })
    }
}
#[derive(Deserialize)]
struct LoginRequestUser<'req> {
    email:    &'req str,
    password: &'req str,
}

async fn login(body: LoginRequest<'_>) -> Result<JSON<UserResponse>, RealWorldError> {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct RegisterRequest<'req> {
    username: &'req str,
    email:    &'req str,
    password: &'req str,
}

async fn register(
    RegisterRequest { username, email, password }: RegisterRequest<'_>,
) -> Result<JSON<UserResponse>, RealWorldError> {
    sqlx::query!(r#"
        SELECT id
        FROM users AS u
        WHERE
            u.name  = $1  AND
            u.email = $2
            -- TODO
    "#,
        username, email
    ).fetch_optional(pool()).await.map_err(RealWorldError::DB)?;

    todo!()
}
