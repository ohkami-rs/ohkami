use ohkami::{
    prelude::{Context, json},
    response::Response,
    result::Result
};
use crate::{
    models::{todo::CreateTodo, repository::TodoRepository},
    TODO_STORE
};

pub(crate) async fn create_todo(c: Context) -> Result<Response> {
    let todo = TODO_STORE.create(c.req.body::<CreateTodo>()?);
    c.Created(json(todo)?)
}

pub(crate) async fn find_todo(c: Context, id: i64) -> Result<Response> {
    todo!()
}

pub(crate) async fn all_todo(c: Context) -> Result<Response> {
    todo!()
}

pub(crate) async fn update_todo(c: Context, id: i64) -> Result<Response> {
    todo!()
}

pub(crate) async fn delete_todo(c: Context, id: i64) -> Result<Response> {
    todo!()
}