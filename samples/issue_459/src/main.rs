use ohkami::{Ohkami, Route};

async fn large_response() -> String {
    (1..=100000)
        .map(|i| format!("This is line #{i}\n"))
        .collect::<String>()
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/".GET(large_response),
    )).run("localhost:3000").await
}
