use ohkami::prelude::*;

fn server() -> Ohkami {
    Ohkami::default()
        .GET("/", || async {Response::OK("Hello!")})
}
fn main() -> Result<()> {
    server().howl(":3000")
}

#[cfg(test)]
mod test {
    use ohkami::{server::Ohkami, response::Response, testing::{Test, Request, Method}};
    use once_cell::sync::Lazy;

    static SERVER: Lazy<Ohkami> = Lazy::new(|| super::server());

    #[test]
    fn test_hello() {
        let request = Request::new(Method::GET, "/");
        SERVER.assert_to_res(&request, Response::OK("Hello!"));
        SERVER.assert_not_to_res(&request, Response::BadRequest(None));
    }
}
