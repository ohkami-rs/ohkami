use ohkami::prelude::*;

async fn health_check(c: Context) -> Response {
    c.NoContent()
}

async fn hello(c: Context, name: String) -> Response {
    c.OK().text(format!("Hello, {name}!"))
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
