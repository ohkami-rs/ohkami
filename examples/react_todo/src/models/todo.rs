use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct Todo {
    pub id:        i32,
    pub text:      String,
    pub completed: bool,
} impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

#[derive(Deserialize, Validate)]
pub(crate) struct CreateTodo {
    #[validate(length(min = 1, max = 100))]
    pub text: String,
}

#[derive(Deserialize, Validate)]
pub(crate) struct UpdateTodo {
    #[validate(length(min = 1, max = 100))]
    pub text:      Option<String>,
    pub completed: Option<bool>,
}
