use async_http::prelude::*;

fn main() -> Context<()> {
    Server::setup()
        .GET("/", hello)
        .GET("/sleepy", sleepy_hello)
        .serve_on(":3000")
}
fn hello<'r>(_: Request<'r>) -> Context<Response> {
    Response::OK(
        JSON::from("hello!")
    )
}
fn sleepy_hello<'r>(_: Request<'r>) -> Context<Response> {
    std::thread::sleep(std::time::Duration::from_secs(5));
    Response::OK(
        JSON::from("hello!")
    )
}
