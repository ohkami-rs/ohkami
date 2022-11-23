use async_http::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/", hello)
        .GET("/sleepy", sleepy_hello)
        .serve_on(":3000")
}

fn hello(_: Context) -> Result<Response> {
    Response::OK(
        JSON::from("hello!")
    )
}
fn sleepy_hello(_: Context) -> Result<Response> {
    std::thread::sleep(std::time::Duration::from_secs(5));
    Response::OK(
        JSON::from("hello!")
    )
}
