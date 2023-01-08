use ohkami::JSON;
use validator::Validate;

#[derive(Clone, PartialEq, Debug)]#[JSON]
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

#[derive(Validate)]#[JSON]
pub(crate) struct CreateTodo {
    #[validate(length(min = 1, max = 100))]
    pub text: String,
}

#[derive(Validate)]#[JSON]
pub(crate) struct UpdateTodo {
    #[validate(length(min = 1, max = 100))]
    pub text:      Option<String>,
    pub completed: Option<bool>,
}
