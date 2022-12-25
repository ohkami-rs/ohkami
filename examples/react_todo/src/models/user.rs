use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct User {
    pub id:   u64,
    pub name: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateUser {
    pub username: String,
}