use std::{collections::HashMap, sync::{Arc, RwLock, RwLockWriteGuard, RwLockReadGuard}};
use ohkami::{result::{Result, ElseResponse}, response::Response};

use super::todo::{Todo, CreateTodo, UpdateTodo};


pub(crate) trait TodoRepository {
    fn create(&self, payload: CreateTodo) -> Todo;
    fn find(&self, id: i32) -> Option<Todo>;
    fn all(&self) -> Vec<Todo>;
    fn update(&self, id: i32, payload: UpdateTodo) -> Result<Todo>;
    fn delete(&self, id: i32) -> Result<()>;
}

pub(crate) struct TodoStore(
    Arc<RwLock<
        HashMap<i32, Todo>
    >>
); impl TodoStore {
    pub fn new() -> Self {
        Self(Arc::default())
    }

    pub fn write_store_ref(&self) -> RwLockWriteGuard<HashMap<i32, Todo>> {
        self.0.write().unwrap(/* --- */)
    }
    pub fn read_store_ref(&self) -> RwLockReadGuard<HashMap<i32, Todo>> {
        self.0.read().unwrap(/* === */)
    }
}

impl TodoRepository for TodoStore {
    fn create(&self, payload: CreateTodo) -> Todo {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32;
        let todo = Todo::new(id, payload.text);
        store.insert(id, todo.clone());
        todo
    }
    fn find(&self, id: i32) -> Option<Todo> {
        self.read_store_ref()
            .get(&id)
            .map(|todo| todo.clone())
    }
    fn all(&self) -> Vec<Todo> {
        Vec::from_iter(
            self.read_store_ref()
                .values()
                .map(|todo| todo.clone())
        )
    }
    fn update(&self, id: i32, payload: UpdateTodo) -> Result<Todo> {
        let mut store = self.write_store_ref();
        let todo = store.get(&id)
            ._else(|| Response::NotFound(format!("Todo of [id == {id}]: NotFound")))?;

        let text = payload.text.unwrap_or(todo.text.clone());
        let completed = payload.completed.unwrap_or(todo.completed);

        let todo = Todo {id, text, completed};
        store.insert(id, todo.clone());
        Ok(todo)
    }
    fn delete(&self, id: i32) -> Result<()> {
        self.write_store_ref()
            .remove(&id)
            ._else(|| Response::NotFound(format!("Todo of [id == {id}]: NotFound")))?;
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use once_cell::sync::Lazy;
    use crate::models::todo::{Todo, CreateTodo, UpdateTodo};
    use super::{TodoStore, TodoRepository};

    static TODO_STORE: Lazy<TodoStore> = Lazy::new(|| TodoStore::new());

    #[test]
    fn todo_crud_senario() {
        let (id, text) = (1, String::from("todo text"));
        let expected = Todo::new(id, text.clone());

        // create
        let todo = TODO_STORE.create(CreateTodo { text });
        assert_eq!(todo, expected);

        // find
        let todo = TODO_STORE.find(todo.id).expect("todo not found");
        assert_eq!(todo, expected);

        // all
        let todo = TODO_STORE.all();
        assert_eq!(todo, vec![expected]);

        // update
        let text = String::from("new todo text");
        let todo = TODO_STORE
            .update(1, UpdateTodo {
                text:      Some(text.clone()),
                completed: Some(true),
            })
            .expect("failed to update todo");
        assert_eq!(todo, Todo { id, text, completed: true });

        // delete
        assert!(TODO_STORE.delete(id).is_ok());
    }
}