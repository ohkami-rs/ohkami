use ohkami::JSON;

#[derive(PartialEq, Debug)]#[JSON]
pub(crate) struct User {
    pub id:   u64,
    pub name: String,
}

#[JSON]
pub(crate) struct CreateUser {
    pub username: String,
}