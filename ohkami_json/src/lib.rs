mod serialize;
mod deserialize;

pub trait JSON: Sized {
    fn serialize(&self) -> String;
    fn deserialize(string: &str) -> Result<Self, &str>;
}

#[cfg(test)]
mod simple_types {
    use crate::JSON;

    #[derive(PartialEq, Debug)]
    struct MyInt(
        usize
    );
    impl JSON for MyInt {
        fn serialize(&self) -> String {
            self.0.to_string()
        }
        fn deserialize(string: &str) -> Result<Self, &str> {
            match string.parse::<usize>() {
                Ok(int) => Ok(Self(int)),
                Err(_) => Err(string)
            }
        }
    }
    #[test]
    fn json_int() {
        let case = MyInt(123);
        assert_eq!(case.serialize(), "123");
        assert_eq!(<MyInt as JSON>::deserialize("123").unwrap(), case);
    }

    #[derive(PartialEq, Debug)]
    struct MyString(
        String
    );
    impl JSON for MyString {
        fn serialize(&self) -> String {
            format!(r#""{}""#, self.0)
        }
        fn deserialize(string: &str) -> Result<Self, &str> {
            if string.starts_with('"')
            && string.ends_with('"') {
                Ok(Self(string[1..string.len()-1].to_owned()))
            } else {
                Err(string)
            }
        }
    }
    #[test]
    fn json_str() {
        let case = MyString(String::from("string!!!"));
        assert_eq!(case.serialize(), r#""string!!!""#);
        assert_ne!(case.serialize(), r#"string!!!"#);
        assert_eq!(<MyString as JSON>::deserialize(r#""string!!!""#).unwrap(), case);
        assert!(<MyString as JSON>::deserialize(r#"string!!!"#).is_err());
    }
}