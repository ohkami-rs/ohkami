#![cfg(test)]

use crate::serde_urlencoded;
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
struct User<'s> {
    name:   Cow<'s, str>,
    age:    Option<Age>,
    gender: Option<Gender>,
}

#[derive(Deserialize, PartialEq, Debug)]
struct URLRequest<'req> {
    url: Cow<'req, str>,
}

#[derive(Deserialize, Debug, PartialEq)]
struct HelloConfig<'req> {
    name: Option<&'req str>,
    repeat: Option<usize>,
}


#[test] fn serialize_struct() {
    assert_eq!(
        serde_urlencoded::to_string(&User {
            name:   Cow::Borrowed("ohkami"),
            age:    None,
            gender: None,
        }).unwrap(),
        "name=ohkami&age=&gender="
    );

    assert_eq!(
        serde_urlencoded::to_string(&User {
            name:   Cow::Owned(String::from("ohkami")),
            age:    None,
            gender: Some(Gender::Other),
        }).unwrap(),
        "name=ohkami&age=&gender=other"
    );

    assert_eq!(
        serde_urlencoded::to_string(&User {
            name:   Cow::Borrowed("ohkami -狼 (おおかみ)-"),
            age:    None,
            gender: Some(Gender::Other),
        }).unwrap(),
        "name=ohkami%20%2D%E7%8B%BC%20%28%E3%81%8A%E3%81%8A%E3%81%8B%E3%81%BF%29%2D&age=&gender=other"
    );
}

#[test] fn deserialize_map() {
    use std::collections::HashMap;

    assert_eq!(
        serde_urlencoded::from_bytes::<HashMap<String, String>>(
            b"key=value"
        ).unwrap(),
        HashMap::from([
            (format!("key"), format!("value")),
        ])
    );

    assert_eq!(
        serde_urlencoded::from_bytes::<HashMap<String, String>>(
            b"key=value&japanese%20%28%E6%97%A5%E6%9C%AC%E8%AA%9E%29=%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF%E3%80%81%E4%B8%96%E7%95%8C%EF%BC%81"
        ).unwrap(),
        HashMap::from([
            (format!("key"), format!("value")),
            (format!("japanese (日本語)"), format!("こんにちは、世界！")),
        ])
    );

    #[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    enum ABCorXYZ {
        ABC, XYZ
    }

    assert_eq!(
        serde_urlencoded::from_bytes::<HashMap<ABCorXYZ, String>>(
            b"ABC=value&XYZ=%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF%E3%80%81%E4%B8%96%E7%95%8C%EF%BC%81"
        ).unwrap(),
        HashMap::from([
            (ABCorXYZ::ABC, format!("value")),
            (ABCorXYZ::XYZ, format!("こんにちは、世界！")),
        ])
    );
}

#[test]
fn deserialize_map_with_empty_fields() {
    use std::collections::HashMap;

    /* empty field to empty string */

    assert_eq!(
        serde_urlencoded::from_bytes::<HashMap<String, String>>(
            b"key="
        ).unwrap(),
        HashMap::from([
            (format!("key"), format!("")),
        ])
    );

    assert_eq!(
        serde_urlencoded::from_bytes::<HashMap<String, String>>(
            b"key1=&key2=value2"
        ).unwrap(),
        HashMap::from([
            (format!("key1"), format!("")),
            (format!("key2"), format!("value2")),
        ])
    );

    assert_eq!(
        serde_urlencoded::from_bytes::<HashMap<String, String>>(
            b"key1=value1&key2="
        ).unwrap(),
        HashMap::from([
            (format!("key1"), format!("value1")),
            (format!("key2"), format!("")),
        ])
    );

    /* empty field to None */

    assert_eq!(
        serde_urlencoded::from_bytes::<HashMap<String, Option<String>>>(
            b"key="
        ).unwrap(),
        HashMap::from([
            (format!("key"), None),
        ])
    );
    
    assert_eq!(
        serde_urlencoded::from_bytes::<HashMap<String, Option<String>>>(
            b"key1=value1&key2="
        ).unwrap(),
        HashMap::from([
            (format!("key1"), Some(format!("value1"))),
            (format!("key2"), None),
        ])
    );
    
    assert_eq!(
        serde_urlencoded::from_bytes::<HashMap<String, Option<String>>>(
            b"key1=&key2=value2"
        ).unwrap(),
        HashMap::from([
            (format!("key1"), None),
            (format!("key2"), Some(format!("value2"))),
        ])
    );
}

#[test] fn deserialize_struct() {
    assert_eq!(
        None,
        serde_urlencoded::from_bytes::<Option<User>>(
            b""
        ).unwrap()
    );
    assert_eq!( 
        User {
            name:   Cow::Borrowed("ohkami"),
            age:    None,
            gender: None,
        },
        serde_urlencoded::from_bytes(
            b"name=ohkami&age=&gender="
        ).unwrap()
    );
    assert_eq!( 
        User {
            name:   Cow::Borrowed("ohkami"),
            age:    None,
            gender: None,
        },
        serde_urlencoded::from_bytes(
            b"age=&name=ohkami&gender="
        ).unwrap()
    );

    assert_eq!(
        User {
            name:   Cow::Owned(String::from("ohkami")),
            age:    None,
            gender: Some(Gender::Other),
        },
        serde_urlencoded::from_bytes(
            b"name=ohkami&age=&gender=other"
        ).unwrap()
    );
    assert_eq!(
        User {
            name:   Cow::Owned(String::from("ohkami")),
            age:    None,
            gender: Some(Gender::Other),
        },
        serde_urlencoded::from_bytes(
            b"gender=other&name=ohkami&age="
        ).unwrap()
    );

    assert_eq!(
        User {
            name:   Cow::Owned(String::from("ohkami -狼 (おおかみ)-")),
            age:    None,
            gender: Some(Gender::Other),
        },
        serde_urlencoded::from_bytes(
            b"name=ohkami%20%2D%E7%8B%BC%20%28%E3%81%8A%E3%81%8A%E3%81%8B%E3%81%BF%29%2D&age=&gender=other"
        ).unwrap()
    );
    assert_eq!(
        User {
            name:   Cow::Owned(String::from("ohkami -狼 (おおかみ)-")),
            age:    None,
            gender: Some(Gender::Other),
        },
        serde_urlencoded::from_bytes(
            b"age=&gender=other&name=ohkami%20%2D%E7%8B%BC%20%28%E3%81%8A%E3%81%8A%E3%81%8B%E3%81%BF%29%2D"
        ).unwrap()
    );

    assert_eq!(
        URLRequest {
            url: Cow::Owned(String::from("https://scrapbox.io/nwtgck/RustのHyper_+_RustlsでHTTPSサーバーを立てるシンプルな例")),
        },
        serde_urlencoded::from_bytes(
            b"url=https://scrapbox.io/nwtgck/Rust%E3%81%AEHyper_+_Rustls%E3%81%A7HTTPS%E3%82%B5%E3%83%BC%E3%83%90%E3%83%BC%E3%82%92%E7%AB%8B%E3%81%A6%E3%82%8B%E3%82%B7%E3%83%B3%E3%83%97%E3%83%AB%E3%81%AA%E4%BE%8B"
        ).unwrap()
    );
}

#[test]
fn unknown_fields() {
    assert_eq!(
        serde_urlencoded::from_bytes::<HelloConfig>(
            b""
        ).unwrap(),
        HelloConfig {
            name: None,
            repeat: None,
        }
    );

    assert_eq!(
        serde_urlencoded::from_bytes::<HelloConfig>(
            b"name=ohkami"
        ).unwrap(),
        HelloConfig {
            name: Some("ohkami"),
            repeat: None,
        }
    );

    assert_eq!(
        serde_urlencoded::from_bytes::<HelloConfig>(
            b"repeat=4"
        ).unwrap(),
        HelloConfig {
            name: None,
            repeat: Some(4),
        }
    );

    assert_eq!(
        serde_urlencoded::from_bytes::<HelloConfig>(
            b"name=ohkami&repeat=4"
        ).unwrap(),
        HelloConfig {
            name: Some("ohkami"),
            repeat: Some(4),
        }
    );

    /* with unknown query fields */

    assert_eq!(
        serde_urlencoded::from_bytes::<HelloConfig>(
            b"unkown=true"
        ).unwrap(),
        HelloConfig {
            name: None,
            repeat: None,
        }
    );

    assert_eq!(
        serde_urlencoded::from_bytes::<HelloConfig>(
            b"name=&unkown=true"
        ).unwrap(),
        HelloConfig {
            name: None,
            repeat: None,
        }
    );

    assert_eq!(
        serde_urlencoded::from_bytes::<HelloConfig>(
            b"name=x&unkown=true"
        ).unwrap(),
        HelloConfig {
            name: Some("x"),
            repeat: None,
        }
    );

    assert_eq!(
        serde_urlencoded::from_bytes::<HelloConfig>(
            b"name=x&unkown="
        ).unwrap(),
        HelloConfig {
            name: Some("x"),
            repeat: None,
        }
    );
}

mod error_case {
    use super::*;

    #[test]
    #[should_panic = "invalid key-value: unexpected end of input"]
    fn unexpected_end_of_input_1() {
        let _ = serde_urlencoded::from_bytes::<HelloConfig>(
            b"name"
        ).unwrap();
    }
    #[test]
    #[should_panic = "invalid key-value: unexpected end of input"]
    fn unexpected_end_of_input_2() {
        let _ = serde_urlencoded::from_bytes::<HelloConfig>(
            b"name=ohkami&age"
        ).unwrap();
    }

    #[test]
    #[should_panic = "invalid key-value: empty key"]
    fn empty_key_1() {
        let _ = serde_urlencoded::from_bytes::<HelloConfig>(
            b"=ohkami"
        ).unwrap();
    }
    #[test]
    #[should_panic = "invalid key-value: empty key"]
    fn empty_key_2() {
        let _ = serde_urlencoded::from_bytes::<HelloConfig>(
            b"name=ohkami&=4"
        ).unwrap();
    }

    #[test]
    #[should_panic = "invalid key-value: missing `=`"]
    fn missing_eq_1() {
        let _ = serde_urlencoded::from_bytes::<HelloConfig>(
            b"ohkami&"
        ).unwrap();
    }
    #[test]
    #[should_panic = "invalid key-value: missing `=`"]
    fn missing_eq_2() {
        let _ = serde_urlencoded::from_bytes::<HelloConfig>(
            b"ohkami&age=4"
        ).unwrap();
    }

    #[test]
    #[should_panic = "invalid key-value: missing `&`"]
    fn missing_amp_1() {
        let _ = serde_urlencoded::from_bytes::<HelloConfig>(
            b"name=ohkami="
        ).unwrap();
    }
    #[test]
    #[should_panic = "invalid key-value: missing `&`"]
    fn missing_amp_2() {
        let _ = serde_urlencoded::from_bytes::<HelloConfig>(
            b"name=ohkami=age=4"
        ).unwrap();
    }
}
