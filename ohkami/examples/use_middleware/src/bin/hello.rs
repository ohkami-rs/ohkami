use ohkami::prelude::*;

fn main() -> Result<()> {
    let middleware = Middleware::new()
        .ANY("/*", |c| async {
            tracing::info!("Hello, middleware!");
            c
        });

    Ohkami::with(middleware)
        .GET("/", || async {
            Response::OK("Hello!")
        })
        .howl("localhost:3000")
}
