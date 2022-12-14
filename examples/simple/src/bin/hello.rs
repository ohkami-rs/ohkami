use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/", |_| async {Response::OK(Body::text("Hello!"))})
        .serve_on(":3000")
}
