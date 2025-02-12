#![allow(unused/* just generating openapi.json for dummy handlers */)]

use ohkami::prelude::*;
use ohkami::openapi;

#[derive(Serialize, openapi::Schema)]
enum Color {
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "green")]
    Green,
}

#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
enum ColorComponent {
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "green")]
    Green,
}

#[derive(Serialize, openapi::Schema)]
enum UserOrTask {
    #[serde(rename = "user")]
    User {
        #[serde(rename = "username")]
        name: String,
        age:  u8,
    },
    #[serde(rename = "task")]
    Task {
        title: String,
        description: Option<String>,
    },
}

#[derive(Serialize, openapi::Schema)]
#[serde(untagged)]
enum UserOrTaskUntagged {
    #[openapi(schema_with = "schema_fn::user")]
    User {
        name: String,
        age:  u8,
    },
    Task {
        title: String,
        description: Option<String>,
    },
}

#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
enum UserOrTaskComponent {
    User {
        name: String,
        age:  u8,
    },
    #[openapi(schema_with = "schema_fn::task")]
    Task {
        title: String,
        description: Option<String>,
    },
}

#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
#[serde(untagged)]
enum UserOrTaskUntaggedComponent {
    #[openapi(schema_with = "schema_fn::user")]
    User {
        name: String,
        age:  u8,
    },
    Task {
        title: String,
        description: Option<String>,
    },
}

#[derive(Serialize, openapi::Schema)]
enum UserOrTaskNewtype {
    User(User),
    Task(Task),
}

#[derive(Serialize, openapi::Schema)]
#[serde(untagged)]
enum UserOrTaskUntaggedNewtype {
    #[openapi(schema_with = "schema_fn::user")]
    User(User),
    Task(Task),
}

#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
enum UserOrTaskNewtypeComponent {
    #[openapi(schema_with = "schema_fn::user")]
    User(User),
    Task(Task),
}

#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
struct User {
    name: String,
    age:  u8,
}

#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
struct Task {
    title:       String,
    description: Option<String>,
}

mod schema_fn {
    use ohkami::openapi;

    pub fn user() -> impl Into<openapi::schema::SchemaRef> {
        openapi::object()
            .property("username", openapi::string())
            .property("age", openapi::integer())
    }

    pub fn task() -> impl Into<openapi::schema::SchemaRef> {
        openapi::object()
            .property("title", openapi::string())
            .optional("body", openapi::string())

    }
}

fn main() {
    macro_rules! dummy_handler {
        ($return_type:ty) => {
            {async fn dummy() -> JSON<$return_type> {todo!()}; dummy}
        };
    }

    let o = Ohkami::new((
        "/color"
            .GET(dummy_handler!(Color))
            .PUT(dummy_handler!(ColorComponent)),
        "/user-or-task"
            .GET(dummy_handler!(UserOrTask))
            .PUT(dummy_handler!(UserOrTaskComponent)),
        "/user-or-task-untagged"
            .GET(dummy_handler!(UserOrTaskUntagged))
            .PUT(dummy_handler!(UserOrTaskUntaggedComponent)),
        "/user-or-task-newtype"
            .GET(dummy_handler!(UserOrTaskNewtype))
            .PUT(dummy_handler!(UserOrTaskNewtypeComponent)),
        "/user-or-task-untagged-newtype"
            .GET(dummy_handler!(UserOrTaskUntaggedNewtype)),
    ));

    o.generate(openapi::OpenAPI {
        title: "Dummy Server for testing #[derive(Schema)] for enums",
        version: "0",
        servers: &[],
    });
}
