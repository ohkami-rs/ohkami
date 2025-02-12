#![allow(unused/* just generating openapi.json for dummy handlers */)]

use ohkami::prelude::*;
use ohkami::serde::json;
use ohkami::openapi;

#[derive(Deserialize, openapi::Schema)]
struct RawCreateUser<'req> {
    name: &'req str,
    age:  u8,
}

#[derive(Deserialize, openapi::Schema, Debug, PartialEq)]
#[serde(from = "RawCreateUser")]
struct CreateUser<'req> {
    username: &'req str,
    age: u8,
}
impl<'req> From<RawCreateUser<'req>> for CreateUser<'req> {
    fn from(raw: RawCreateUser<'req>) -> Self {
        Self {
            username: raw.name,
            age: raw.age,
        }
    }
}

#[derive(Deserialize, openapi::Schema)]
#[serde(try_from = "RawCreateUser")]
struct ValidatedCreateUser<'req> {
    username: &'req str,
    age:  u8,
}
impl<'req> TryFrom<RawCreateUser<'req>> for ValidatedCreateUser<'req> {
    type Error = String;

    fn try_from(raw: RawCreateUser<'req>) -> Result<Self, Self::Error> {
        if raw.age < 18 {
            return Err(format!("User's age must be 18 or more"))
        }

        Ok(Self {
            username: raw.name,
            age: raw.age,
        })
    }
}

#[derive(Serialize, openapi::Schema, Clone)]
#[serde(into = "UserResponse")]
struct User {
    name: String,
    age:  u8,
}

#[derive(Serialize, openapi::Schema)]
struct UserResponse {
    user: UserResponseUser,
}
#[derive(Serialize, openapi::Schema)]
struct UserResponseUser {
    name: String,
    age: u8,
}
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            user: UserResponseUser {
                name: user.name,
                age: user.age,
            }
        }
    }
}

fn main() {
    macro_rules! dummy_handler {
        (-> $return_type:ty) => {
            {async fn dummy() -> JSON<$return_type> {todo!()}; dummy}
        };
        ($req_type:ty) => {
            {async fn dummy(_: JSON<$req_type>) {}; dummy}
        };
    }

    let o = Ohkami::new((
        "/from".GET(dummy_handler!(CreateUser<'_>)),
        "/try_from".GET(dummy_handler!(ValidatedCreateUser<'_>)),
        "/into".GET(dummy_handler!(-> UserResponse)),
    ));

    assert!(Result::is_err(&json::from_str::<CreateUser>(r#"{
        "username": "ohkami",
        "age": 4
    }"#)));
    assert_eq!(json::from_str::<CreateUser>(r#"{
        "name": "ohkami",
        "age": 4
    }"#).unwrap(), CreateUser {
        username: "ohkami",
        age: 4
    });

    let u = User {
        name: format!("ohkami"),
        age:  4
    };
    assert_ne!(
        json::to_string(&u).unwrap(),
        r#"{"name":"ohkami","age":4}"#
    );
    assert_eq!(
        json::to_string(&u).unwrap(),
        r#"{"user":{"name":"ohkami","age":4}}"#
    );

    o.generate(openapi::OpenAPI {
        title: "Dummy Server for serde From & Into",
        version: "0",
        servers: &[],
    });
}
