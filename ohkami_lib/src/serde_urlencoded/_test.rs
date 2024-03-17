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
}

// #[test] fn deserialize_struct() {
//     assert_eq!( 
//         User {
//             name:   Cow::Borrowed("ohkami"),
//             age:    None,
//             gender: None,
//         },
//         serde_urlencoded::from_str(
//             "name=ohkami&age=&gender="
//         ).unwrap()
//     );
// 
//     assert_eq!(
//         User {
//             name:   Cow::Owned(String::from("ohkami")),
//             age:    None,
//             gender: Some(Gender::Other),
//         },
//         serde_urlencoded::from_str(
//             "name=ohkami&age=&gender=other&k="
//         ).unwrap()
//     );
// 
//     assert_eq!(
//         User {
//             name:   Cow::Owned(String::from("ohkami -狼 (おおかみ)-")),
//             age:    None,
//             gender: Some(Gender::Other),
//         },
//         serde_urlencoded::from_str(
//             "name=ohkami%20%2D%E7%8B%BC%20%28%E3%81%8A%E3%81%8A%E3%81%8B%E3%81%BF%29%2D&age=&gender=other"
//         ).unwrap()
//     );
// }
// 