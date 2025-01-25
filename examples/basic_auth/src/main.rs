use ohkami::prelude::*;
use ohkami::fang::BasicAuth;

#[tokio::main]
async fn main() {
    let private_ohkami = Ohkami::new((
        BasicAuth {
            username: "master of hello",
            password: "world"
        },
        "/hello".GET(|| async {"Hello, private :)"})
    ));

    Ohkami::new((
        "/hello".GET(|| async {"Hello, public!"}),
        "/private".By(private_ohkami)
    )).howl("localhost:8888").await
}
