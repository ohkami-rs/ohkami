use ohkami::prelude::*;
use ohkami::sse::DataStream;
use tokio::time::{sleep, Duration};

async fn handler() -> DataStream {
    DataStream::new(|mut s| async move {
        s.send("starting streaming...");
        for i in 1..=5 {
            sleep(Duration::from_secs(1)).await;
            s.send(format!("MESSAGE #{i}"));
        }
        s.send("streaming finished!");
    })
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/sse".GET(handler),
    )).howl("localhost:3020").await
}
