use ohkami::prelude::*;

async fn large_response() -> String {
    (1..=100000)
        .map(|i| format!("This is line #{i}\n"))
        .collect::<String>()
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        // route("/").get(large_response),
        "/".GET(large_response),
    )).howl("localhost:3000").await
}
