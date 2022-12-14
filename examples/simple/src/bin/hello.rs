use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/", |_| async {Response::OK(Body::text("Hello!"))})
        .GET("/sleepy/:time", sleepy_hello)
        .serve_on(":3000")
}

async fn sleepy_hello(ctx: Context) -> Result<Response> {
    let sleep_time = ctx.param()
        ._else(|| Response::BadRequest("Expected sleeping duration as path parameter."))?;
    (sleep_time < 30)
        ._else(|| Response::BadRequest("Sorry, please request a sleeping duration (sec) less than 30."))?;
    
    std::thread::sleep(std::time::Duration::from_secs(sleep_time as u64));

    Response::OK(Body::text("Hello, I'm sleepy..."))
}
