#![allow(unused)]
use ohkami::Payload;

#[Payload(JSON)]
#[derive(serde::Deserialize)]
struct CreateUserRequest1 {
    name:     String,
    password: String,
    age:      u8,
}

// #[Payload(JSON)]
// #[derive(serde::Deserialize)]
// struct CreateUserRequest2<'name, 'password> {
//     name:     &'name str,
//     password: &'password str,
//     age:      Option<u8>,
// }

#[Payload(URLEncoded)]
struct UpdateUserRequest1 {
    name:     String,
    password: String,
    age:      u8,
}

#[Payload(URLEncoded)]
struct UpdateUserRequest2 {
    name:     String,
    password: String,
    age:      Option<u8>,
}

// #[Payload(URLEncoded)]
// struct UpdateUserRequest3<'name, 'password> {
//     name:     &'name str,
//     password: &'password str,
//     age:      Option<u8>,
// }

fn main() {}
