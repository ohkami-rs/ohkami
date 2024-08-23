use ohkami::prelude::*;
use ohkami::format::JSON;

#[derive(Serialize)]
struct Message {
    message: String
}

async fn hello(name: &str) -> JSON<Message> {
    JSON(Message {
        message: format!("Hello, {name}!")
    })
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/hello/:name".GET(hello),
    )).howl("localhost:3000").await
}
