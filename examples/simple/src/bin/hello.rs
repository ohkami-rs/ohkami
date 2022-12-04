use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/", hello)
        .GET("/sleepy/:time", sleepy_hello)
        .serve_on(":3000")
}

fn hello(_: Context) -> Result<Response> {
    Response::OK(
        JSON::from("Hello!")
    )
}

fn sleepy_hello(ctx: Context) -> Result<Response> {
    let sleep_time = ctx.path_param
        .else_response(|| Response::BadRequest("Expected sleeping duration as path parameter."))?;
    (sleep_time < 30)
        .else_response(|| Response::BadRequest("Sorry, please request a sleeping duration (sec) less than 30."))?;
    
    std::thread::sleep(std::time::Duration::from_secs(sleep_time as u64));

    Response::OK(
        JSON::from("Hello, I'm sleepy...")
    )
}
