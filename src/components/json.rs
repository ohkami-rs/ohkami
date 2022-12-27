use std::{fmt::Debug, ops::Deref};
use serde::{Serialize, Deserialize};
use crate::{result::Result, response::body::Body};


#[derive(Debug, PartialEq, Clone)]
pub enum JSON<T: Serialize + for <'d> Deserialize<'d>> {
    Ser(String),
    De(T),
} impl<T: Serialize + for <'d> Deserialize<'d>> JSON<T> {
    pub fn ser(self) -> Result<String> {
        match self {
            Self::Ser(s) => Ok(s),
            Self::De(d) => Ok(serde_json::to_string(&d)?),
        }
    }
    pub fn de(self) -> Result<T> {
        match self {
            Self::De(d) => Ok(d),
            Self::Ser(s) => Ok(serde_json::from_str(&s)?),
        }
    }
}

impl<T: Serialize + for <'d> Deserialize<'d>> Deref for JSON<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::De(t) => t,
            Self::Ser(_) => unimplemented!(),
        }
    }
}


pub fn json<T: Serialize>(value: T) -> Result<Body> {
    Ok(Body::application_json(
        serde_json::to_string(&value)?
    ))
}

/// Utility macro to create `Body::application_json` value from some pair(s) of key-value(s).
/// ```no_run
/// let result = json!{"ok": true};
/// ```
/// ```no_run
/// let res = json!{"token": "abcxyz", "expires": "2022-01-01 00:00"};
/// ```
#[macro_export]
macro_rules! json {
    {$($key:literal : $value:expr),*} => {
        Body::application_json(
            String::from("{")
            $( + &format!(",\"{}\":{:?}", $key, $value) )*
            + "}"
        )
    };
}
