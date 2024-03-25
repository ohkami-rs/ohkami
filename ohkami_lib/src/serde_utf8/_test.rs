use crate::serde_utf8;


#[test] fn serialize_bool() {
    assert_eq!(serde_utf8::to_string(&true).unwrap(), "true");
    assert_eq!(serde_utf8::to_string(&false).unwrap(), "false");
}

#[test] fn serialize_newtype() {
    #[derive(serde::Serialize)]
    struct MyText(String);

    assert_eq!(
        serde_utf8::to_string(
            &MyText(String::from("Hello, serde!"))
        ).unwrap(),
        "Hello, serde!"
    );
    
    #[derive(serde::Serialize)]
    struct MyCowText(std::borrow::Cow<'static, str>);

    assert_eq!(
        serde_utf8::to_string(
            &MyCowText(std::borrow::Cow::Borrowed("Hello, serde!"))
        ).unwrap(),
        "Hello, serde!"
    );
}

#[test] fn serialize_enum() {
    #![allow(dead_code)]
    
    #[derive(serde::Serialize)]
    enum Color { Red, Blue, Green }

    assert_eq!(
        serde_utf8::to_string(&Color::Blue).unwrap(),
        "Blue"
    );

    #[derive(serde::Serialize)]
    enum Color2 {
        #[serde(rename = "red")]
        Red,
        #[serde(rename = "blue")]
        Blue,
        #[serde(rename = "green")]
        Green,
    }

    assert_eq!(
        serde_utf8::to_string(&Color2::Blue).unwrap(),
        "blue"
    );
}

#[test] fn serialize_non_newtype_struct_makes_err() {
    #[derive(serde::Serialize)]
    struct User {
        id:   usize,
        name: String,
    }

    assert!(
        serde_utf8::to_string(&User {
            id:   42,
            name: String::from("ohkami"),
        }).is_err()
    );
}


#[test] fn deserialize_bool() {
    assert_eq!(
        serde_utf8::from_str::<bool>("true").unwrap(),
        true
    );

    assert!(
        serde_utf8::from_str::<bool>("unknown").is_err(),
    );
}

#[test] fn deserialize_newtype() {
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct MyText(String);

    assert_eq!(
        serde_utf8::from_str::<MyText>("Hello, serde!").unwrap(),
        MyText(String::from("Hello, serde!"))
    );
    
    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct MyCowText(std::borrow::Cow<'static, str>);

    assert_eq!(
        serde_utf8::from_str::<MyCowText>("Hello, serde!").unwrap(),
        MyCowText(std::borrow::Cow::Borrowed("Hello, serde!"))
    );
}

#[test] fn deserialize_non_newtype_struct_makes_err() {
    #[derive(serde::Deserialize)]
    struct User {
        _id:   usize,
        _name: String,
    }

    assert!(
        serde_utf8::from_str::<User>("hogefuga").is_err()
    );
}
