use crate::serde_urlencoded;
use std::borrow::Cow;
use ::serde_derive::{Serialize, Deserialize};


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
        serde_urlencoded::from_str::<HashMap<String, String>>(
            "key=value"
        ).unwrap(),
        HashMap::from([
            (format!("key"), format!("value")),
        ])
    );

    assert_eq!(
        serde_urlencoded::from_str::<HashMap<String, String>>(
            "key=value&japanese%20%28%E6%97%A5%E6%9C%AC%E8%AA%9E%29=%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF%E3%80%81%E4%B8%96%E7%95%8C%EF%BC%81"
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
        serde_urlencoded::from_str::<HashMap<ABCorXYZ, String>>(
            "ABC=value&XYZ=%E3%81%93%E3%82%93%E3%81%AB%E3%81%A1%E3%81%AF%E3%80%81%E4%B8%96%E7%95%8C%EF%BC%81"
        ).unwrap(),
        HashMap::from([
            (ABCorXYZ::ABC, format!("value")),
            (ABCorXYZ::XYZ, format!("こんにちは、世界！")),
        ])
    );
}

#[test] fn deserialize_struct() {
    assert_eq!( 
        User {
            name:   Cow::Borrowed("ohkami"),
            age:    None,
            gender: None,
        },
        serde_urlencoded::from_str(
            "name=ohkami&age=&gender="
        ).unwrap()
    );

    assert_eq!(
        User {
            name:   Cow::Owned(String::from("ohkami")),
            age:    None,
            gender: Some(Gender::Other),
        },
        serde_urlencoded::from_str(
            "name=ohkami&age=&gender=other"
        ).unwrap()
    );

    assert_eq!(
        User {
            name:   Cow::Owned(String::from("ohkami -狼 (おおかみ)-")),
            age:    None,
            gender: Some(Gender::Other),
        },
        serde_urlencoded::from_str(
            "name=ohkami%20%2D%E7%8B%BC%20%28%E3%81%8A%E3%81%8A%E3%81%8B%E3%81%BF%29%2D&age=&gender=other"
        ).unwrap()
    );
}
