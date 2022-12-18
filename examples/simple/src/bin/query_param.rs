use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/annoying_hello", annoying_hello)
        .serve_on(":3000")
}

async fn annoying_hello(ctx: Context) -> Result<Response> {
    let count = ctx.query("count")?
        .parse::<usize>()
        ._else(|_| Response::BadRequest("Expected `count` to be a interger."))?;
    (count < 10)
        ._else(|| Response::BadRequest("Sorry, `count` must be less than 10."))?;

    let name = ctx.query("name")?;
        
    Response::OK(format!(
        "Hello, {}!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!", &name).repeat(count)
    )
}
