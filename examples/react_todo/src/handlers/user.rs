use ohkami::{
    context::Context,
    result::Result,
    response::Response,
    components::json::json, prelude::JSON,
};
use crate::models::user::{User, CreateUser};


pub(crate) async fn create_user(c: Context, payload: JSON<CreateUser>) -> Result<Response> {
    let name = payload.de()?.username;
    let user = User {
        id: 1337,
        name,
    };
    c.Created(json(user))
}