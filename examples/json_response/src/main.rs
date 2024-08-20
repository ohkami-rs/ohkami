use ohkami::{Ohkami, Route};
use ohkami::format::JSON;
use ohkami::serde::Serialize;


#[derive(Serialize)]
struct User {
    id:   u64,
    name: String,
}

async fn single_user() -> JSON<User> {
    JSON(User {
        id:   42,
        name: String::from("ohkami"),
    })
}

async fn multiple_users() -> JSON<Vec<User>> {
    JSON(vec![
        User {
            id:   42,
            name: String::from("ohkami"),
        },
        User {
            id:   1024,
            name: String::from("bynari"),
        }
    ])
}

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/single"  .GET(single_user),
        "/multiple".GET(multiple_users),
    )).howl("localhost:5000").await
}
