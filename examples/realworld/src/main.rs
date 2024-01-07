mod config;
mod models;
mod repositories;
mod fangs;
mod handler;


#[tokio::main]
async fn main() {
    handler::realworld_ohkami()
        .howl(":8080").await
}
