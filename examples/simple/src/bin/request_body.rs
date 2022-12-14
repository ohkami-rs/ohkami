use ohkami::{prelude::*, json};
use serde::Deserialize;

fn main() -> Result<()> {
    Server::setup()
        .POST("/api/login", only_first_user_can_login)
        .serve_on(":3000")
}

#[derive(Deserialize)]
struct User {
    id:    i64,
    _name: String,
}

async fn only_first_user_can_login(ctx: Context) -> Result<Response> {
    let requested_user = ctx.body::<User>()
        ._else(|err| err.error_context("can't deserialize user"))?;
    (requested_user.id == 1)
        ._else(|| Response::Forbidden(
            "Noooo!! Only first user of this service can login by this endpoint!"
        ))?;
    Response::OK(json!("ok": true))
}