use ohkami::macros::JSON;

#[derive(JSON, PartialEq, Debug)]
pub(crate) struct User {
    pub id:   u64,
    pub name: String,
}

#[derive(JSON)]
pub(crate) struct CreateUser {
    pub username: String,
}