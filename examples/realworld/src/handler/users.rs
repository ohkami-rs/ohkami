use ohkami::{Ohkami, Route, Context, Response};
use ohkami::utils::Payload;
use serde::Deserialize;


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
    email:    String,
    password: String,
}

async fn login(c: Context, body: LoginRequest) -> Response {
    todo!()
}

#[Payload(JSON)]
#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email:    String,
    password: String,
}

async fn register(c: Context, body: RegisterRequest) -> Response {
    todo!()
}
