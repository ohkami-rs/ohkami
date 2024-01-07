mod config;
mod errors;
mod models;
mod repositories;
mod fangs;
mod handler;


#[tokio::main]
async fn main() {
    handler::realworld_ohkami()
        .howl(":8080").await
}
