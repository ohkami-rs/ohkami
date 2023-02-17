mod models;
mod handlers;

use handlers::{root, user::create_user, todo::{create_todo, all_todo, find_todo, delete_todo, update_todo}};
use models::repository::TodoStore;

use once_cell::sync::Lazy;
use ohkami::{
    server::Ohkami, prelude::Config,
    error::Result,
};

static TODO_STORE: Lazy<TodoStore> = Lazy::new(|| TodoStore::new());

fn server() -> Ohkami {
    let config = Config {
        log_subscribe: Some(
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
        ),
        ..Default::default()
    };

    Ohkami::with(config)
        .GET("/", root)
        .POST("/users", create_user)

        .GET( "/todos", all_todo)
        .POST("/todos", create_todo)
        
        .GET(   "/todos/:id", find_todo)
        .DELETE("/todos/:id", delete_todo)
        .PATCH( "/todos/:id", update_todo)
}

fn main() -> Result<()> {
    server().howl(":3000")
}

#[cfg(test)]
mod test {
    use crate::models::{user::User, todo::Todo};

    use once_cell::sync::Lazy;
    use ohkami::{
        testing::{Test, Request, Method::*, Status},
        response::Response, server::Ohkami,
    };

    static SERVER: Lazy<Ohkami> = Lazy::new(|| super::server());

    #[test]
    fn should_return_hello_world() {
        SERVER.assert_to_res(
            &Request::new(GET, "/"),
            Response::OK("Hello, World!")
        )
    }

    #[test]
    fn should_return_user_data() {
        let req = Request::new(POST, "/users")
            .header("Content-Type", "application/json")
            .body(r#"{ "username": "Taro" }"#);
        let res = SERVER.oneshot_json::<User>(&req);

        assert_eq!(res, User {
            id:   1337,
            name: "Taro".into(),
        })
    }

    #[test]
    fn todo_http_crud_senario() {
        let sample_todo_1 = Todo::new(1, String::from("sample todo text 1"));
        let sample_todo_2 = Todo::new(2, String::from("sample todo text 2"));


        // too short (empty) text
        let req = Request::new(POST, "/todos")
            .header("Content-Type", "application/json")
            .body(r#"{ "text": "" }"#);
        let res = SERVER.oneshot_res(&req);
        assert_eq!(res.status, Status::BadRequest);

        // too long (longer than 100) text
        let long_text = "1234567890".repeat(12);
        let req = Request::new(POST, "/todos")
            .header("Content-Type", "application/json")
            .body(format!(r#"{{ "text": "{long_text}" }}"#));
        let res = SERVER.oneshot_res(&req);
        assert_eq!(res.status, Status::BadRequest);
        

        // create
        let req = Request::new(POST, "/todos")
            .header("Content-Type", "application/json")
            .body(r#"{ "text": "sample todo text 1" }"#);
        let res = SERVER.oneshot_json::<Todo>(&req);
        assert_eq!(res, sample_todo_1);

        // find
        let req = Request::new(GET, "/todos/1");
        let res = SERVER.oneshot_json::<Todo>(&req);
        assert_eq!(res, sample_todo_1);

        // all
        SERVER.oneshot_res(
            &Request::new(POST, "/todos")
                .header("Content-Type", "application/json")
                .body(r#"{ "text": "sample todo text 2" }"#)
        );
        let req = Request::new(GET, "/todos");
        let mut res = SERVER.oneshot_json::<Vec<Todo>>(&req);
        res.sort_by_key(|todo| todo.id); //
        assert_eq!(res, vec![sample_todo_1, sample_todo_2]);

        // update
        let req = Request::new(PATCH, "/todos/1")
            .header("Content-Type", "application/json")
            .body(r#"{ "text": "updated text" }"#);
        let res = SERVER.oneshot_json::<Todo>(&req);
        assert_eq!(res, Todo::new(1, String::from("updated text")));

        // delete
        let req = Request::new(DELETE, "/todos/1");
        let res = (*SERVER).oneshot_res(&req);
        assert_eq!(res.status, Status::OK);
    }
}
