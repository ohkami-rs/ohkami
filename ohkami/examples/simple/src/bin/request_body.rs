use ohkami::{prelude::*, json};


fn main() -> Result<()> {
    Ohkami::default()
        .POST("/api/login_j", only_whose_name_starts_with_j_can_login)
        .howl(":3000")
}

#[JSON]
#[allow(unused)]
struct User {
    name:     String,
    password: String,
}

async fn only_whose_name_starts_with_j_can_login(payload: User) -> Result<Response> {
    (payload.name.starts_with('j'))
        ._else(|| Response::Forbidden(
            "Noooo!! Only first user whose name starts with 'j' can login by this endpoint!"
        ))?;
    Response::OK(json! {"ok": true})
}

#[cfg(test)]
mod test {
    use ohkami::{prelude::*, json, testing::{Test, Request, Method}};
    use super::{only_whose_name_starts_with_j_can_login, User};

    #[test]
    fn test_api_login_j() {
        let server = Ohkami::default()
            .POST("/api/login_j", only_whose_name_starts_with_j_can_login);

        server.assert_to_res(
            &Request::new(Method::POST, "/api/login_j")
                .body(User {
                    name:     "jTaro".into(),
                    password: "iamjtaro".into(),
                }),
            Response::OK(json! {"ok": true})
        )
    }
}