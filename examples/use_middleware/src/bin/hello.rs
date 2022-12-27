use ohkami::prelude::*;

fn main() -> Result<()> {
    let middleware = Middleware::new()
        .ANY("/*", |c| async {
            tracing::info!("Hello, middleware!");
            c
        });

    Server::with(middleware)
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .serve_on("localhost:3000")
}
