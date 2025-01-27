use ohkami::prelude::*;
use ohkami::sse::DataStream;
use ohkami::openapi::{OpenAPI, Server};

#[tokio::main]
async fn main() {
    let o = Ohkami::new((
        "/once".GET(hello_once),
        "/".GET(intervally_hello)
    ));

    o.generate(OpenAPI {
        title:   "Streaming Sample API",
        version: "0.1.0",
        servers: &[Server::at("http://localhost:8080")]
    });

    o.howl("localhost:8080").await
}

async fn hello_once() -> &'static str {
    "Hello!"
}

async fn intervally_hello() -> DataStream<&'static str> {
    DataStream::new(|mut s| async move {
        for _ in 0..5 {
            s.send("Hello!");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        s.send("Bye!");
    })
}
