use ohkami::prelude::*;

async fn sleeping_hello(secs: u64) -> &'static str {
    tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
    "Hello, sleep!"
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/sleep/:secs".GET(sleeping_hello),
    )).howl("localhost:8888").await
}
