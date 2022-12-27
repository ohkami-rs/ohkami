use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::default()
        .GET("/", || async {Response::OK("Hello!")})
        .serve_on(":3000")
}
