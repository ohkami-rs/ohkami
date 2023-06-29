mod handler {
    use ohkami::{Context, Response, Query};

    pub async fn health_check(c: Context) -> Response {
        c.NoContent()
    }

    #[Query]
    pub struct HelloQuery<'name> {
        name:   &'name str,
        repeat: Option<usize>,
    }

    pub async fn hello<'q>(c: Context, query: HelloQuery<'q>) -> Response<String> {
        let HelloQuery { name, repeat } = query;
        let message = format!("Hello, {name}!").repeat(repeat.unwrap_or(0));
        c.Text(message)
    }
}

mod server {
    use ohkami::{Ohkami, Route};
    use crate::handler as h;

    pub async fn serve() {
        Ohkami::new()(
            "/hc"
                .GET(h::health_check),
            "/hello"
                .GET(h::hello),
        ).howl(3000).await
    }
}

fn main() {
    async_std::task::block_on(server::serve())
}
