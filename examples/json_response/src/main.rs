use ohkami::{typed::ResponseBody, Ohkami, Route};

#[ResponseBody(JSONS)]
struct User {
    id:   u64,
    name: String,
}

async fn single_user() -> User {
    User {
        id:   42,
        name: String::from("ohkami"),
    }
}

async fn multiple_users() -> Vec<User> {
    vec![
        User {
            id:   42,
            name: String::from("ohkami"),
        },
        User {
            id:   1024,
            name: String::from("bynari"),
        }
    ]
}

async fn nullable_user() -> Option<User> {
    None
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/single"  .GET(single_user),
        "/multiple".GET(multiple_users),
        "/nullable".GET(nullable_user),
    )).howl("localhost:5000").await
}
