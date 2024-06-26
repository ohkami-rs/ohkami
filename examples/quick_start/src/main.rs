use ohkami::prelude::*;
use ohkami::typed::status;

async fn health_check() -> status::NoContent {
    status::NoContent
}

async fn hello(name: &str) -> String {
    format!("Hello, {name}!")
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/healthz"
            .GET(health_check),
        "/hello/:name"
            .GET(hello),
    )).howl("localhost:3000").await
}
