use ohkami::{Ohkami, Route, utils::{Payload, JSON}};
use serde::Deserialize;
use crate::models::{User, UserResponse};


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

async fn login(body: LoginRequest) -> JSON<UserResponse> {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email:    String,
    password: String,
}

async fn register(body: RegisterRequest) -> JSON<UserResponse> {
    todo!()
}
