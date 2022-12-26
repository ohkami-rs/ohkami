mod models;
mod handlers;

use handlers::{root, user::create_user, todo::{create_todo, all_todo, find_todo, delete_todo, update_todo}};
use models::repository::TodoStore;

use once_cell::sync::Lazy;
use ohkami::{
    server::Server, prelude::Config,
    result::Result,
};

static TODO_STORE: Lazy<TodoStore> = Lazy::new(|| TodoStore::new());

fn server() -> Server {
    let config = Config {
        log_subscribe: Some(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
        ),
        ..Default::default()
    };

    Server::setup_with(config)
        .GET("/", root)

        .POST("/users", create_user)

        .GET( "/todos", all_todo)
        .POST("/todos", create_todo)
        
        .GET(   "/todos/:id", find_todo)
        .DELETE("/todos/:id", delete_todo)
        .PATCH( "/todos/:id", update_todo)
}

fn main() -> Result<()> {
    server().serve_on(":3000")
}

#[cfg(test)]
mod test {
    use crate::{models::{user::User, todo::Todo}, TODO_STORE};

    use once_cell::sync::Lazy;
    use ohkami::{
        test::{Test, Request, Method::*},
        response::Response, server::Server,
    };

    static SERVER: Lazy<Server> = Lazy::new(|| super::server());

    #[test]
    fn should_return_hello_world() {
        (*SERVER).assert_to_res(
            &Request::new(GET, "/"),
            Response::OK("Hello, World!")
        )
    }

    #[test]
    fn should_return_user_data() {
        let req = Request::new(POST, "/users")
            .body(r#"{ "username": "Taro" }"#);

        let res = (&SERVER).oneshot_json(&req)
            .to_struct::<User>()
            .expect("request body isn't User");

        assert_eq!(res, User {
            id:   1337,
            name: "Taro".into(),
        })
    }

    #[test]
    fn should_create_todo() {
        let expected = Todo::new(1, String::from("should_create_todo"));

        let mut lock = TODO_STORE.write_store_ref();
        lock.clear();

        let req = Request::new(POST, "/todos")
            .body(r#"{ "text": "should_create_todo" }"#);
        let res = (&SERVER).oneshot_json(&req)
            .to_struct::<Todo>()
            .expect("response isn't a Todo");

        assert_eq!(res, expected)
    }

    #[test]
    fn should_find_todo() {
        let expected = Todo::new(1, String::from("should_find_todo"));

        let mut lock = TODO_STORE.write_store_ref();
        lock.clear();
        lock.insert(1, expected.clone());

        let req = Request::new(GET, "/todos/1");
        let res = (&SERVER).oneshot_json(&req)
            .to_struct::<Todo>()
            .expect("response isn't a Todo");
        
        assert_eq!(res, expected)
    }

    #[test]
    fn should_get_all_todos() {
        let expected = vec![
            Todo::new(1, String::from("should_get_all_todo_1")),
            Todo::new(2, String::from("should_get_all_todo_2")),
        ];

        let mut lock = TODO_STORE.write_store_ref();
        lock.clear();
        lock.insert(1, expected[0].clone());
        lock.insert(2, expected[1].clone());

        let req = Request::new(GET, "/todos");
        let res = (*SERVER).oneshot_json(&req)
            .to_struct::<Vec<Todo>>()
            .expect("response isn't a Vec<Todo>");

        assert_eq!(res, expected);
    }

    #[test]
    fn should_update_todo() {
        let expected = Todo::new(1, String::from("should_update_todo"));
        let before = Todo::new(1, String::from("before_update_todo"));

        let mut lock = TODO_STORE.write_store_ref();
        lock.clear();
        lock.insert(1, before);

        let req = Request::new(PATCH, "/todos/1")
            .body(r#"{ "text": "should_update_todo" }"#);
        let res = (*SERVER).oneshot_json(&req)
            .to_struct::<Todo>()
            .expect("response isn't a Todo");

        assert_eq!(res, expected)
    }

    #[test]
    fn should_delete_todo() {
        let mut lock = TODO_STORE.write_store_ref();
        lock.clear();
        lock.insert(1, Todo::new(1, String::from("should_delete_todo")));

        let req = Request::new(DELETE, "/todos/1");
        let res = (*SERVER).oneshot_res(&req);
    }
}
