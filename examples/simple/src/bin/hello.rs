use ohkami::prelude::*;

fn main() -> Result<()> {
    Ohkami::default()
        .GET("/", || async {Response::OK("Hello!")})
        .howl(":3000")
}
