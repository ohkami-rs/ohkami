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
#[derive(Deserialize)]
struct LoginRequest {
    user: LoginRequestUser,
}

#[derive(Deserialize)]
struct LoginRequestUser {
    email:    String,
    password: String,
}

async fn login(body: LoginRequest) -> Result<JSON<UserResponse>, RealWorldError> {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email:    String,
    password: String,
}

async fn register(
    RegisterRequest { username, email, password }: RegisterRequest,
) -> Result<JSON<UserResponse>, RealWorldError> {
    // sqlx::query!(r#"
    //     SELECT id
    //     FROM users AS u
    //     WHERE
    //         u.name = $1  AND
    //         u.email = $2 AND
    //         u.pa
    // "#, username, email).fetch_optional(pool()).await.map_err(RealWorldError::DB)?;

    todo!()
}
