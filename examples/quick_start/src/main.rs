use ohkami::{Ohkami, Route};
use ohkami::claw::{status, Path};

async fn health_check() -> status::NoContent {
    status::NoContent
}

async fn hello(Path(name): Path<&str>) -> String {
    format!("Hello, {name}!")
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/healthz"
            .GET(health_check),
        "/hello/:name"
            .GET(hello),
    )).run("localhost:3000").await
}
