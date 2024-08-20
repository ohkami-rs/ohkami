use ohkami::prelude::*;
use ohkami::{serde::*, format::JSON};

#[cfg(feature="DEBUG")]
#[derive(Clone)]
struct Logger;
#[cfg(feature="DEBUG")]
impl FangAction for Logger {
    async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
        tracing::info!("\n{req:?}");
        Ok(())
    }
    async fn back<'a>(&'a self, res: &'a mut Response) {
        tracing::info!("\n{res:?}");
    }
}

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
    #[cfg(feature="DEBUG")]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    #[cfg(feature="DEBUG")]
    let fangs = Logger;

    #[cfg(not(feature="DEBUG"))]
    let fangs = ();

    Ohkami::with(fangs, (
        "/hello/:name".GET(hello),
    )).howl("localhost:3000").await
}
