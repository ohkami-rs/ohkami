use ohkami::prelude::*;
use ohkami::http::{Status, Text};

async fn health_check() -> Status {
    Status::NoContent
}

async fn hello(name: &str) -> Text {
    Text::OK(format!("Hello, {name}!"))
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/hc".
            GET(health_check),
        "/hello/:name".
            GET(hello),
    )).howl(3000).await
}
