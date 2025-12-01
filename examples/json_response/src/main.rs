use ohkami::{Ohkami, Route};
use ohkami::claw::Json;
use ohkami::serde::Serialize;


#[derive(Serialize)]
struct User {
    id:   u64,
    name: String,
}

async fn single_user() -> Json<User> {
    Json(User {
        id:   42,
        name: String::from("ohkami"),
    })
}

async fn multiple_users() -> Json<Vec<User>> {
    Json(vec![
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
    )).run("localhost:5000").await
}
