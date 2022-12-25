use ohkami::prelude::*;

fn main() -> Result<()> {
    let middleware = Middleware::new()
        .ANY("/*", middleware::cors)
        .ANY("/api/*", middleware::hello);

    Server::setup_with(middleware)
        .GET("/api", handler::hello)
        .GET("/api/sleepy/:time", handler::sleepy_hello)
        .serve_on("localhost:3000")
}

mod middleware {
    use ohkami::{prelude::*, components::headers::AdditionalHeader::*};

    pub async fn hello(c: Context) -> Context {
        tracing::debug!("Hello, middleware!");
        c
    }

    pub async fn cors(mut c: Context) -> Context {
        c.header(AccessControlAllowOrigin, "localhost:8000");
        tracing::debug!("by cors(): context: {c:#?}");
        c
    } 
}

mod handler {
    use ohkami::prelude::*;

    pub async fn hello(c: Context) -> Result<Response> {
        c.OK("Hello!")
    }

    pub async fn sleepy_hello(c: Context, time: u64) -> Result<Response> {
        (time < 10)
            ._else(|| c.BadRequest("`time` must be less than 10"))?;
        std::thread::sleep(
            std::time::Duration::from_secs(time)
        );
        c.OK("Hello, I'm sleepy...")
    }
}
