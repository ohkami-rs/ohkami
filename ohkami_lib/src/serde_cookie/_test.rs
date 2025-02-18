#![cfg(test)]

use crate::serde_cookie;
use std::borrow::Cow;
use ::serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Age(u8);

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Gender {
    #[serde(rename = "male")]
    Male,
    #[serde(rename = "female")]
    Felmale,
    #[serde(rename = "other")]
    Other,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct UserInfo<'s> {
    name:   Cow<'s, str>,
    age:    Option<Age>,
    gender: Option<Gender>,
}

#[test]
fn simple_ascii_cookies() {
    assert_eq!(
        serde_cookie::from_str::<UserInfo>(
            "name=ohkami; age=4"
        ).unwrap(),
        UserInfo {
            name: Cow::Borrowed("ohkami"),
            age: Some(Age(4)),
            gender: None,
        }
    );

    assert_eq!(
        serde_cookie::from_str::<UserInfo>(
            "age=4; name=ohkami; gender=other"
        ).unwrap(),
        UserInfo {
            name: Cow::Borrowed("ohkami"),
            age: Some(Age(4)),
            gender: Some(Gender::Other),
        }
    );
}

#[test]
fn simple_ascii_cookies_with_double_quoted_values() {
    assert_eq!(
        serde_cookie::from_str::<UserInfo>(
            r#"name="ohkami"; age=4"#
        ).unwrap(),
        UserInfo {
            name: Cow::Borrowed("ohkami"),
            age: Some(Age(4)),
            gender: None,
        }
    );

    assert_eq!(
        serde_cookie::from_str::<UserInfo>(
            r#"age=4; name="ohkami"; gender="other""#
        ).unwrap(),
        UserInfo {
            name: Cow::Borrowed("ohkami"),
            age: Some(Age(4)),
            gender: Some(Gender::Other),
        }
    );
}

#[test]
fn nonascii_encoded_cookies() {
    assert_eq!(
        serde_cookie::from_str::<UserInfo>(
            "name=%E7%8B%BC; age=4"
        ).unwrap(),
        UserInfo {
            name: Cow::Borrowed("狼"),
            age: Some(Age(4)),
            gender: None,
        }
    );

    assert_eq!(
        serde_cookie::from_str::<UserInfo>(
            "age=4; name=\"%E7%8B%BC\"; gender=other"
        ).unwrap(),
        UserInfo {
            name: Cow::Borrowed("狼"),
            age: Some(Age(4)),
            gender: Some(Gender::Other),
        }
    );
}
