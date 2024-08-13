use ohkami::prelude::*;
use ohkami::fang::BasicAuth;

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/hello".GET(|| async {"Hello, public!"}),
        "/private".By(Ohkami::with(
            BasicAuth {
                username: "master of hello",
                password: "world"
            },
            "/hello".GET(|| async {"Hello, private :)"})
        ))
    )).howl("localhost:8888").await
}
