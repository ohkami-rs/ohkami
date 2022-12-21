use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/", || async {Response::OK("Hello!")})
        .serve_on(":3000")
}
