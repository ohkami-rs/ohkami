use ohkami::prelude::*;
use ohkami::handle::{Path, Json};

#[derive(Serialize)]
struct Message {
    message: String
}

async fn hello(Path(name): Path<&str>) -> Json<Message> {
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
