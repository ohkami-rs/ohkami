use ohkami::prelude::*;
use ohkami::{typed::Payload, builtin::payload::JSON};

struct Logger;
const _: () = {
    impl<I: FangProc> Fang<I> for Logger {
        type Proc = LoggerProc<I>;
        fn chain(&self, inner: I) -> Self::Proc {
            LoggerProc { inner }
        }
    }

    struct LoggerProc<I: FangProc> {
        inner: I
    }
    impl<I: FangProc> FangProc for LoggerProc<I> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            tracing::info!("\n{req:?}");
            let res = self.inner.bite(req).await;
            tracing::info!("\n{res:?}");
            res
        }
    }
};

#[Payload(JSON/S)]
struct Message {
    message: String
}

async fn hello(name: &str) -> Message {
    Message {
        message: format!("Hello, {name}!")
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    Ohkami::with(Logger, (
        "/hello/:name".GET(hello),
    )).howl("localhost:3000").await
}
