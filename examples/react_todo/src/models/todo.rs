use serde::{Deserialize, Serialize};

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

#[derive(Deserialize)]
pub(crate) struct CreateTodo {
    pub text: String,
}

pub(crate) struct UpdateTodo {
    pub text:      Option<String>,
    pub completed: Option<bool>,
}
