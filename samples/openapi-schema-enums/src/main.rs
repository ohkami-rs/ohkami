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
#[serde(untagged)]
enum UserOrTaskUntagged {
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
    Task {
        title: String,
        description: Option<String>,
    },
}

#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
#[serde(untagged)]
enum UserOrTaskUntaggedComponent {
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
    User(User),
    Task(Task),
}

#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
enum UserOrTaskNewtypeComponent {
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
