use ohkami::prelude::*;

fn server() -> Server {
    Server::setup()
        .GET("/", |_| async {Response::OK("Hello!")})
}
fn main() -> Result<()> {
    server().serve_on(":3000")
}

#[cfg(test)]
mod test {
    use ohkami::{server::Server, response::Response, test_system::{Request, Method}};
    use once_cell::sync::Lazy;

    static SERVER: Lazy<Server> = Lazy::new(|| super::server());

    #[test]
    fn test_hello() {
        let request = Request::new(Method::GET, "/");
        (*SERVER).assert_to_res(&request, Response::OK("Hello!"));
        (*SERVER).assert_not_to_res(&request, Response::BadRequest(None));
    }
}
