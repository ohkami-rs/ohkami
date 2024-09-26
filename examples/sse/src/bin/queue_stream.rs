use ohkami::util::{stream, StreamExt};
use tokio::time::sleep;


#[tokio::main]
async fn main() {
    let mut qs = stream::queue(|mut q| async move {
        for i in 1..=5 {
            sleep(std::time::Duration::from_secs(1)).await;
            q.push(format!("Hi, I'm message#{i}!"))
        }
    });
    
    while let Some(message) = qs.next().await {
        println!("{message}")
    }
}
