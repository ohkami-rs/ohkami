use std::{iter::Peekable, str::Chars};

mod serialize;
mod deserialize;

pub trait JSON: Sized {
    fn serialize(&self) -> String;
    fn deserialize(string: &str) -> Option<Self> {
        Self::_deserialize(&mut string.chars().peekable())
    }
    fn _deserialize(string: &mut Peekable<Chars>) -> Option<Self>;
}

#[cfg(test)]
mod test {
    use std::{iter::Peekable, str::Chars};
    use crate::{JSON, serialize::Serialize, deserialize::Deserialize};

    #[derive(Debug, PartialEq)]
    struct User {
        id:   u64,
        name: String,
    }
    impl JSON for User {
        fn serialize(&self) -> String {
            format!(r#"{{"id":{},"name":{}}}"#,
                <u64 as Serialize>::serialize(&self.id),
                <String as Serialize>::serialize(&self.name),
            )
        }
        fn _deserialize(string: &mut Peekable<Chars>) -> Option<Self> {
            let (mut id, mut name) = (None, None);

            string.next_if_eq(&'{')?;
            loop {
                match string.peek()? {
                    '}' => {
                        string.next();
                        return (
                            string.next().is_none() &&
                            id.is_some() &&
                            name.is_some()
                        ).then(|| User {
                            id:   id.unwrap(),
                            name: name.unwrap(),
                        })
                    },
                    _ => match <String as Deserialize>::_deserialize(string)?.as_str() {
                        "id" => {
                            string.next_if_eq(&':')?;
                            string.next_if_eq(&' ');
                            if id.replace(<u64 as Deserialize>::_deserialize(string)?)
                                .is_some() {return None}
                            string.next_if_eq(&',');
                            string.next_if_eq(&' ');
                        },
                        "name" => {
                            string.next_if_eq(&':')?;
                            string.next_if_eq(&' ');
                            if name.replace(<String as Deserialize>::_deserialize(string)?)
                                .is_some() {return None}
                            string.next_if_eq(&',');
                            string.next_if_eq(&' ');
                        },
                        _ => return None,
                    },
                }
            }
        }
    }
    #[test] #[allow(unused_labels)]
    fn json_user() {
        let case = User {
            id:   1,
            name: String::from("abc"),
        };

        'serialize: {
            assert_eq!(case.serialize(), r#"{"id":1,"name":"abc"}"#);
        }
        
        'deserialize: {
            let mut string = r#"{"id":1,"name":"abc"}"#.chars().peekable();
            let d = <User as JSON>::_deserialize(&mut string);
            assert_eq!(string.collect::<String>(), "");
            assert_eq!(d.as_ref(), Some(&case));

            let mut string = r#"{"id": 1, "name": "abc"}"#.chars().peekable();
            let d = <User as JSON>::_deserialize(&mut string);
            assert_eq!(string.collect::<String>(), "");
            assert_eq!(d.as_ref(), Some(&case));

            assert!(<User as JSON>::deserialize(r#"{id:1,"name":"abc"}"#).is_none());
            assert!(<User as JSON>::deserialize(r#"{"id":"1","name":"abc"}"#).is_none());
            assert!(<User as JSON>::deserialize(r#"{"id":1}"#).is_none());
            assert!(<User as JSON>::deserialize(r#"{"id":1, "name":"abc", "id":2}"#).is_none());
        }
    }
}