use ohkami::{prelude::*, group::*};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    id:   usize,
    name: String,
}

fn main() -> Result<()> {
    Ohkami::default()
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .route("/api",
            GET(hello_api).POST(reflect)
        )
        .howl(":3000")
}

async fn hello_api() -> Result<Response> {
    Response::OK("Hello, api!")
}

async fn reflect(payload: JSON<User>) -> Result<Response> {
    Response::OK(payload)
}