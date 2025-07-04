use ohkami::prelude::*;
use ohkami::format::Json;

#[derive(Serialize)]
struct Message {
    message: String
}

async fn hello(name: &str) -> Json<Message> {
    Json(Message {
        message: format!("Hello, {name}!")
    })
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/hello/:name".GET(hello),
    )).howl("localhost:3000").await
}
