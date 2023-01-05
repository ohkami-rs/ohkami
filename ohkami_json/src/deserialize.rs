use std::{str::Chars, iter::Peekable};

pub trait Deserialize: Sized {
    fn deserialize(string: &str) -> Option<Self> {
        <Self as Deserialize>::_deserialize(&mut string.chars().peekable())
    }
    fn _deserialize(string: &mut Peekable<Chars>) -> Option<Self>;
}

impl Deserialize for String {
    fn _deserialize(string: &mut Peekable<Chars>) -> Option<Self> {
        string.next_if_eq(&'"')?;
        let mut ret = String::new();
        while let Some(ch) = string.next() {
            match ch {
                '"' => return Some(ret),
                _ => ret.push(ch),
            }
        }
        None
    }
}

impl Deserialize for bool {
    fn _deserialize(string: &mut Peekable<Chars>) -> Option<Self> {
        match string.next() {
            Some('t') => Some(
                string.next() == Some('r') &&
                string.next() == Some('u') &&
                string.next() == Some('e')
            ),
            Some('f') => Some(!(
                string.next() == Some('a') &&
                string.next() == Some('l') &&
                string.next() == Some('s') &&
                string.next() == Some('e')
            )),
            _ => None
        }
    }
}

macro_rules! impl_for_int {
    ($( $int:ty )*) => {
        $(
            impl Deserialize for $int {
                fn _deserialize(string: &mut Peekable<Chars>) -> Option<Self> {
                    let mut int_str = String::new();
                    while let Some(ch) = string.peek() {
                        match ch {
                            '0'..='9' => int_str.push(string.next().unwrap()),
                            _ => break,
                        }
                    }
                    int_str.parse::<$int>().ok()
                } 
            }
        )*
    };
} impl_for_int!(
    u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize
);

impl<D: Deserialize> Deserialize for Vec<D> {
    fn _deserialize(string: &mut Peekable<Chars>) -> Option<Self> {
        string.next_if_eq(&'[')?;
        let mut ret = Vec::new();
        loop {
            match string.peek() {
                Some(']') => {string.next(); return Some(ret)},
                Some(' ') => {string.next();},
                _ => {
                    ret.push(<D as Deserialize>::_deserialize(string)?);
                    string.next_if_eq(&',');
                }
            }
        }
    }
}


#[cfg(test)]
mod test {
    use crate::deserialize::Deserialize;

    #[test]
    fn deserialize_int() {
        assert_eq!(
            <u8 as Deserialize>::deserialize("123").unwrap(),
            123
        );
    }

    #[test]
    fn deserialize_string() {
        assert_eq!(
            <String as Deserialize>::deserialize(r#""string!!!""#).unwrap(),
            String::from("string!!!")
        );
        assert!(<String as Deserialize>::deserialize("string!!!").is_none());
        assert!(<String as Deserialize>::deserialize(r#""string!!!"#).is_none());
    }

    #[test]
    fn deserialize_bool() {
        assert_eq!(
            <bool as Deserialize>::deserialize("true").unwrap(),
            true
        );
        assert_eq!(
            <bool as Deserialize>::deserialize("false").unwrap(),
            false
        );
    }

    #[test]
    fn deserialize_vec() {
        assert_eq!(
            <Vec<u8> as Deserialize>::deserialize("[]").unwrap(),
            vec![]
        );
        {
            let mut case = "[1,2,3]@".chars().peekable();
            <Vec<u8> as Deserialize>::_deserialize(&mut case);
            assert_eq!(case.collect::<String>(), "@");
        }
        assert_eq!(
            <Vec<u8> as Deserialize>::deserialize("[1,2,3]").unwrap(),
            vec![1,2,3]
        );
        assert_eq!(
            <Vec<u8> as Deserialize>::deserialize("[1, 2, 3]").unwrap(),
            vec![1,2,3]
        );
        assert!(<Vec<u8> as Deserialize>::deserialize("").is_none());
        assert!(<Vec<u8> as Deserialize>::deserialize("[").is_none());
        assert!(<Vec<u8> as Deserialize>::deserialize("[1").is_none());
        assert!(<Vec<u8> as Deserialize>::deserialize("[1,").is_none());

        assert_eq!(
            <Vec<String> as Deserialize>::deserialize(r#"[""]"#).unwrap(),
            vec![""]
        );
        assert_eq!(
            <Vec<String> as Deserialize>::deserialize(r#"["a", "b", "c"]"#).unwrap(),
            vec!["a","b","c"]
        );
    }
}