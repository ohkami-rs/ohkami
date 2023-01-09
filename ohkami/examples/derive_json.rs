use ohkami::macros::JSON;

#[derive(PartialEq, JSON)]
struct User {
    id: u64,
    name: String,
}

fn main() {}
