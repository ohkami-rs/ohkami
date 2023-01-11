use ohkami::prelude::*;

fn main() -> Result<()> {
    let middleware = Middleware::new()
        .beforeANY("/*", |c| async {
            tracing::info!("Hello, middleware!");
            c
        });

    Ohkami::with(middleware)
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .GET("/night", || async {
            Response::OK("Good morning!")
        })
        .howl("localhost:3000")
}
