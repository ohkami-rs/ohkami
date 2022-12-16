use ohkami::{prelude::*, json};
use serde::Deserialize;

fn main() -> Result<()> {
    Server::setup()
        .POST("/api/login_j", only_whos_name_starts_with_j_can_login)
        .serve_on(":3000")
}

#[derive(Deserialize)]
struct User {
    name:     String,
    password: String,
}

async fn only_whos_name_starts_with_j_can_login(ctx: Context) -> Result<Response> {
    let requested_user = ctx.body::<User>()
        ._else(|err| err.error_context("can't deserialize user"))?;
    (requested_user.name.starts_with('j'))
        ._else(|| Response::Forbidden(
            "Noooo!! Only first user whose name starts with 'j' can login by this endpoint!"
        ))?;
    Response::OK(json!("ok": true))
}