use ohkami_macros::{NewJSON, consume_struct};

#[derive(NewJSON)]
struct User {
    id:   u64,
    name: String,
}

fn main() {}
