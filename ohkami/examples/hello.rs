#![allow(unused)]
fn main() {}

mod handlers {
    use ohkami::{Context, Response, Query};

    pub async fn health_check(c: Context) -> Response {
        c.NoContent()
    }

    #[Query]
    pub struct HelloQuery {
        name:   String,
        repeat: Option<usize>,
    }

    pub async fn hello(c: Context, query: HelloQuery) -> Response<String> {
        let HelloQuery { name, repeat } = query;
        let message = format!("Hello, {name}!").repeat(repeat.unwrap_or(1));
        c.Text(message)
    }
}

mod server {
    use ohkami::{Ohkami, Route};
    use crate::handlers as h;

    pub async fn serve() {
        Ohkami::new()(
            "/hc"
                .GET(h::health_check),
            "/hello"
                .GET(h::hello),
        ).howl(3000).await
    }
}
