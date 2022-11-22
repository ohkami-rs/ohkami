use async_http::prelude::*;

fn main() -> Context<()> {
    Server::setup()
        .GET("/", hello)
        .GET("/sleepy", sleepy_hello)
        .serve_on(":3000")
}
fn hello(_: Request) -> Context<Response> {
    Response::OK(
        JSON::from("hello!")
    )
}
fn sleepy_hello(_: Request) -> Context<Response> {
    std::thread::sleep(std::time::Duration::from_secs(5));
    Response::OK(
        JSON::from("hello!")
    )
}
