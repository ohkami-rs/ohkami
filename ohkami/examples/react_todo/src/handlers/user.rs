use ohkami::{
    context::Context,
    error::Result,
    response::Response,
};
use crate::models::user::{User, CreateUser};


pub(crate) async fn create_user(c: Context, payload: CreateUser) -> Result<Response> {
    let user = User {
        id: 1337,
        name: payload.username,
    };
    c.Created(user)
}