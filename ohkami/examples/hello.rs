use ohkami::{
    Context, Response, Ohkami, Route,
    Query,
};


async fn health_check(c: Context) -> Response {
    c.NoContent()
}


#[Query]
struct HelloQuery<'name> {
    name:   &'name str,
    repeat: Option<usize>,
}

async fn hello<'q>(c: Context, query: HelloQuery<'q>) -> Response<String> {
    let HelloQuery { name, repeat } = query;
    let message = format!("Hello, {name}!").repeat(repeat.unwrap_or(0));
    c.Text(message)
}


fn main() {
    async fn serve() {
        Ohkami::new()(
            "/hc"
                .GET(health_check),
            "/hello"
                .GET(hello),
        ).howl(3000).await
    }

    async_std::task::block_on(serve())
}
