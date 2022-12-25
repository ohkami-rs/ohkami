use ohkami::{prelude::*, json};
use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
    Server::setup()
        .POST("/api/login_j", only_whose_name_starts_with_j_can_login)
        .serve_on(":3000")
}

#[derive(Deserialize, Serialize)]
#[allow(unused)]
struct User {
    name:     String,
    password: String,
}

async fn only_whose_name_starts_with_j_can_login(ctx: Context) -> Result<Response> {
    let requested_user = ctx.req.body::<User>()
        ._else(|err| err.error_context("can't deserialize user"))?;
    (requested_user.name.starts_with('j'))
        ._else(|| Response::Forbidden(
            "Noooo!! Only first user whose name starts with 'j' can login by this endpoint!"
        ))?;
        
    Response::OK(json!("ok": true))
}

#[cfg(test)]
mod test {
    use ohkami::{prelude::*, json, test::{Test, Request, Method}};
    use super::{only_whose_name_starts_with_j_can_login, User};

    #[test]
    fn test_api_login_j() {
        let server = Server::setup()
            .POST("/api/login_j", only_whose_name_starts_with_j_can_login);

        server.assert_to_res(
            &Request::new(Method::POST, "/api/login_j")
                .body(User {
                    name:     "jTaro".into(),
                    password: "iamjtaro".into(),
                }),
                // .body("{name:jTaro, password:iamjtaro}"),
            Response::OK(json!("ok": true))
        )
    }
}