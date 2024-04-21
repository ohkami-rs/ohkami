use ohkami::{Ohkami, Route};
use ohkami::{typed::Payload, builtin::payload::JSON};


#[Payload(JSON/S)]
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

#[tokio::main]
async fn main() {
    Ohkami::new((
        "/single"  .GET(single_user),
        "/multiple".GET(multiple_users),
    )).howl("localhost:5000").await
}
