use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/annoying_hello", annoying_hello)
        .serve_on(":3000")
}

async fn annoying_hello(ctx: Context) -> Result<Response> {
    let name: &str = ctx.query("name")?;
    let count = ctx.query("count")?;
    (count < 10)
        ._else(|| Response::BadRequest("Sorry, `count` must be less than 10."))?;
    Response::OK(format!(
        "Hello, {}!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!", &name).repeat(count)
    )
}
