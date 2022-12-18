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
    use ohkami::server::Server;
    use once_cell::sync::Lazy;

    static SERVER: Lazy<Server> = Lazy::new(|| super::server());

    #[test]
    fn test_hello() {
        
    }
}
