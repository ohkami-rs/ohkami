use ohkami::typed::DataStream;
use ohkami::util::StreamExt;


#[tokio::main]
async fn main() {
    let mut iter = (1..=5).map(|i| async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Result::<_, std::convert::Infallible>::Ok(format!(
            "I'm message #{i} !"
        ))
    });
    while let Some(d) = iter.next() {
        let d = d.await;
        println!("d = {}", d.unwrap())
    }

    let mut ds = DataStream::from_iter_async((1..=5).map(|i| async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Result::<_, std::convert::Infallible>::Ok(format!(
            "I'm message #{i} !"
        ))
    }));
    while let Some(d) = ds.next().await {
        println!("d = {}", d.unwrap())
    }
}
