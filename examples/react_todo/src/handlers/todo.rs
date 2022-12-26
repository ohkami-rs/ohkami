use ohkami::{
    prelude::{Context, json},
    response::Response,
    result::{Result, ElseResponse, ElseResponseWithErr},
    components::json::JSON, json
};
use validator::Validate;
use crate::{
    models::{todo::{CreateTodo, UpdateTodo}, repository::TodoRepository},
    TODO_STORE
};

pub(crate) async fn create_todo(c: Context) -> Result<Response> {
    let payload = c.req.body::<CreateTodo>()?;
    payload.validate()
        ._else(|e| c.BadRequest(format!("Invalid request: {}", e.to_string())))?;
    let todo = TODO_STORE.create(payload);
    c.Created(json(todo)?)
}

pub(crate) async fn find_todo(c: Context, id: i32) -> Result<Response> {
    let todo = TODO_STORE.find(id)
        ._else(|| c.NotFound("Todo not found"))?;
    c.OK(json(todo)?)
}

pub(crate) async fn all_todo(c: Context) -> Result<Response> {
    let todos = TODO_STORE.all();
    c.OK(json(todos)?)
}

pub(crate) async fn update_todo(c: Context, id: i32) -> Result<Response> {
    let payload = c.req.body::<UpdateTodo>()?;
    let updated = TODO_STORE.update(id, payload)?;
    c.OK(json(updated)?)
}

pub(crate) async fn delete_todo(c: Context, id: i32) -> Result<Response> {
    TODO_STORE.delete(id)?;
    c.OK(json!("ok": true))
}