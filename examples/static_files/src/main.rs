use ohkami::prelude::*;

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/static-files".Dir("./public").omit_extensions(["html"]),
    )).howl("localhost:3000").await
}
