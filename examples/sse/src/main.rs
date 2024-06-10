use ohkami::prelude::*;
use ohkami::typed::DataStream;


#[tokio::main]
async fn main() {
    Ohkami::new((
        "/sse".GET(sse),
    )).howl("localhost:5050").await
}

async fn sse() -> DataStream<String> {
    DataStream::from_iter_async((1..=5).map(|i| async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        Result::<_, std::convert::Infallible>::Ok(format!(
            "I'm message #{i} !"
        ))
    }))
}
