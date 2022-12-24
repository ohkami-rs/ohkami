use ohkami::prelude::*;

fn main() -> Result<()> {
    let middleware = Middleware::init()
        .ANY("/*", || async {
            tracing::info!("Hello, middleware!")
        });

    Server::setup_with(middleware)
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .serve_on("localhost:3000")
}
