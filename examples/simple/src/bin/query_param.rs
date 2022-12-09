use ohkami::prelude::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    Server::setup()
        .GET("/annoying_hello", annoying_hello)
        .serve_on(":3000")
}

async fn annoying_hello(ctx: Context) -> Result<Response> {
    let count = ctx.query("count")
        .else_response(|| Response::BadRequest("Expected query parameter `count`."))?
        .parse::<usize>()
        .else_response(|_| Response::BadRequest("Expected `count` to be a interger."))?;
    (count < 10)
        .else_response(|| Response::BadRequest("Sorry, `count` must be less than 10."))?;
    let name = ctx.query("name")
        .else_response(|| Response::BadRequest("Expected query parameter `name`."))?;
    
    let message = format!("Hello, {}!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!", &name).repeat(count);
    Response::OK(JSON::from(message))
}
