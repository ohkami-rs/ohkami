mod config;
mod errors;
mod models;
mod fangs;
mod handlers;


#[tokio::main]
async fn main() -> Result<(), errors::RealWorldError> {
    config::init().await?;

    // handlers::realworld_ohkami()
    //     .howl(":8080").await;
    for (k, v) in std::env::vars() {
        println!("[env] {k} = {v}")
    }

    Ok(())
}
