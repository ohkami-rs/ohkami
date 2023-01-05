use std::{fmt::Debug, ops::Deref};
use serde::{Serialize, Deserialize};
use crate::{result::Result, response::body::Body};


pub trait Json<'j>: Serialize + Deserialize<'j> {
    fn ser(self) -> String;
    fn de(string: &str) -> Self;
}

#[derive(Debug, PartialEq, Clone)]
pub enum JSON<T: Serialize + for <'d> Deserialize<'d>> {
    Ser(String),
    De(T),
} impl<T: Serialize + for <'d> Deserialize<'d>> JSON<T> {
    /// ```no_run
    /// {
    ///     match self {
    ///         Self::Ser(s) => Ok(s),
    ///         Self::De(d) => Ok(serde_json::to_string(&d)?),
    ///     }
    /// }
    /// ```
    pub fn ser(self) -> Result<String> {
        match self {
            Self::Ser(s) => Ok(s),
            Self::De(d) => Ok(serde_json::to_string(&d)?),
        }
    }

    /// ```no_run
    /// {
    ///     match self {
    ///         Self::De(d) => Ok(d),
    ///         Self::Ser(s) => Ok(serde_json::from_str(&s)?),
    ///     }
    /// }
    /// ```
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

/// Try serializing the value.
/// ```no_run
/// c.OK(json(user))
/// ```
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
    {$key1:literal : $value1: expr $(, $key:literal : $value:expr)*} => {
        Body::application_json(
            String::from("{")
            + &format!("\"{}\":{:?}", $key1, $value1)
            $( + &format!(",\"{}\":{:?}", $key, $value) )*
            + "}"
        )
    };
}



#[cfg(test)]
mod test {
    use serde::{Serialize, Deserialize};

    use crate::prelude::{Response, Result};

    use super::JSON;

    #[derive(Serialize, Deserialize)]
    struct User {
        id:   i64,
        name: String,
    }

    async fn _h1(payload: JSON<User>) -> Result<Response> {
        Response::Created(payload)
    }
    async fn _h2(payload: JSON<User>) -> Result<Response> {
        let _user: User = payload.de()?;
        Response::NoContent()
    }
}

// #[cfg(test)]
// mod test_json {
//     use super::Json;
//     use ohkami_macros::Json;
//     use serde::{Serialize, Deserialize};

//     #[derive(Json, Serialize, Deserialize, Debug, PartialEq)]
//     struct User {
//         id:   u64,
//         name: String,
//     }

//     #[test]
//     fn json_user() {
//         let sample_user = User {
//             id: 1,
//             name: "jsoner".into()
//         };
//         assert_eq!(<User as Json>::de(r#"
//             {
//                 "id": 1,
//                 "name": "jsoner"
//             }
//         "#), sample_user);
//         assert_eq!(
//             sample_user.ser(),
//             String::from(r#"{"id":1,"name":"jsoner"}"#)
//         );
//     }
// }