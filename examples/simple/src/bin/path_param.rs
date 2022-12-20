use ohkami::prelude::*;

fn main() -> Result<()> {
    Server::setup()
        .GET("/sleepy/:time", sleepy_hello)
        .serve_on(":3000")
}

async fn sleepy_hello(_: Context, time: usize) -> Result<Response> {
    (time < 30)
        ._else(|| Response::BadRequest("Sorry, please request a sleeping duration (sec) less than 30."))?;
    
    std::thread::sleep(
        std::time::Duration::from_secs(time as u64)
    );

    Response::OK("Hello, I'm sleepy...")
}
