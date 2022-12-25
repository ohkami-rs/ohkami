use ohkami::{
    context::Context,
    result::Result,
    response::Response,
    components::json::json,
};
use crate::models::user::{User, CreateUser};


pub(crate) async fn create_user(c: Context) -> Result<Response> {
    let name = c.req.body::<CreateUser>()?.username;
    let user = User {
        id: 1337,
        name,
    };
    c.Created(json(user)?)
}