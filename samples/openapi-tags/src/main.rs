#![allow(unused/* just for demo of `openapi::Tag` */)]

use ohkami::prelude::*;
use ohkami::typed::status;
use ohkami::openapi;

mod users {
    use super::*;

    pub(super) fn ohkami() -> Ohkami {
        Ohkami::new((
            openapi::Tag("users"),
            "/"
                .GET(list_users)
                .POST(create_user),
            "/:id"
                .GET(get_user_profile),
        ))
    }

    #[derive(Serialize, openapi::Schema)]
    struct User {
        id:   i32,
        name: String,
        age:  Option<u8>,
    }

    #[derive(Deserialize, openapi::Schema)]
    struct CreateUser<'req> {
        name: &'req str,
        age:  Option<u8>,
    }

    async fn list_users() -> JSON<Vec<User>> {
        JSON(vec![])
    }

    async fn create_user(
        JSON(req): JSON<CreateUser<'_>>,
    ) -> status::Created<JSON<User>> {
        status::Created(JSON(User {
            id:   42,
            name: req.name.into(),
            age:  req.age
        }))
    }

    async fn get_user_profile(id: i32) -> JSON<User> {
        JSON(User {
            id,
            name: "unknown".into(),
            age:  Some(42)
        })
    }
}

mod tasks {
    use super::*;

    pub(super) fn ohkami() -> Ohkami {
        Ohkami::new((
            openapi::Tag("tasks"),
            "/list"
                .GET(list_tasks),
            "/:id/edit"
                .PUT(edit_task),
        ))
    }

    #[derive(Serialize, openapi::Schema)]
    struct Task {
        id:          i32,
        title:       String,
        #[openapi(schema_with = "description_schema")]
        description: String,
    }

    #[derive(Deserialize, openapi::Schema)]
    struct EditTask<'req> {
        title:       Option<&'req str>,
        description: Option<&'req str>,
    }

    async fn list_tasks() -> JSON<Vec<Task>> {
        JSON(vec![])
    }

    async fn edit_task(
        JSON(req): JSON<EditTask<'_>>
    ) -> status::NoContent {
        status::NoContent
    }

    fn description_schema() -> impl Into<openapi::SchemaRef> {
        openapi::string()
            .format("Japanese")
    }
}

fn main() {
    let users_ohkami = users::ohkami();
    let tasks_ohkami = tasks::ohkami();

    let api_ohkami = Ohkami::new((
        openapi::Tag("api"),
        "/"
            .GET(|| async {"Hello, tags!"}),
        "/users".By(users_ohkami),
        "/tasks".By(tasks_ohkami),
    ));

    let o = Ohkami::new((
        "/health"
            .GET(|| async {status::NoContent}),
        "/api".By(api_ohkami),
    ));

    o.generate(openapi::OpenAPI {
        title:   "Sample API",
        version: "0.0.0",
        servers: &[openapi::Server::at("http://localhost:6666")]
    })
}
