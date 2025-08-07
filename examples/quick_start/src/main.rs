use ohkami::{Ohkami, Route};
use ohkami::handle::{status, Path};

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
    )).howl("localhost:3000").await
}
