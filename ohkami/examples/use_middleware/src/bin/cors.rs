use ohkami::prelude::*;

fn main() -> Result<()> {
    let middleware = Middleware::new()
        .beforeANY("*", |c| async {
            tracing::info!("got request!");
            c
        })
        .beforeANY("/api/*", middleware::hello)
        .afterANY("/*", middleware::cors);

    Ohkami::with(middleware)
        .GET("/", handler::hello)
        .GET("/api", handler::hello)
        .GET("/api/sleepy/:time", handler::sleepy_hello)
        .howl("localhost:3000")
}

mod middleware {
    use ohkami::prelude::*;
    use Header::AccessControlAllowOrigin;

    pub async fn hello(c: Context) -> Context {
        tracing::debug!("Hello, middleware!");
        c
    }

    pub async fn cors(mut res: Response) -> Response {
        res.add_header(AccessControlAllowOrigin, "localhost:8000");
        res
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
