use ohkami::prelude::*;
use ohkami::{typed::Payload, builtin::payload::JSON};

#[derive(Clone)]
struct Logger;
impl FangAction for Logger {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        tracing::info!("\n{req:?}");
        Ok(())
    }
    async fn back<'a>(&'a self, res: &'a mut Response) {
        tracing::info!("\n{res:?}");
    }
}

#[Payload(JSON/S)]
struct Message {
    message: String
}

async fn hello(name: &str) -> Message {
    Message {
        message: format!("Hello, {name}!")
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    Ohkami::with(Logger, (
        "/hello/:name".GET(hello),
    )).howl("localhost:3000").await
}
