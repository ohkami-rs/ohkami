mod models;
mod handlers;

use handlers::{root, user::create_user};
use ohkami::{
    result::Result,
    server::Server, prelude::Config,
};

fn server() -> Server {
    let config = Config {
        log_subscribe: Some(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
        ),
        ..Default::default()
    };

    Server::setup_with(config)
        .GET("/",       root)
        .POST("/users", create_user)
}

fn main() -> Result<()> {
    server().serve_on(":3000")
}

#[cfg(test)]
mod test {
    use once_cell::sync::Lazy;
    use crate::models::user::User;
    use ohkami::{
        test::{Test, Request, Method},
        response::Response, server::Server,
    };

    static SERVER: Lazy<Server> = Lazy::new(|| super::server());

    #[test]
    fn should_return_hello_world() {
        (*SERVER).assert_to_res(
            &Request::new(Method::GET, "/"),
            Response::OK("Hello, World!")
        )
    }

    #[test]
    fn should_return_user_data() {
        let req = Request::new(Method::POST, "/users")
                .body("{\"username\": \"Taro\"}");

        let res = (&SERVER).oneshot_json(&req)
            .to_struct::<User>()
            .expect("request body isn't User");

        assert_eq!(res, User {
            id:   1337,
            name: "Taro".into(),
        })
    }
}
