use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use crate::result::Result;


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

/*

/// Try serializing a struct implementing `serde::Serialize` and return `Result<JSON>`.
/// ```no_run
/// #[derive(Serialize)]
/// struct User {
///     id:   i64,
///     name: String,
/// }
/// 
/// async fn handler(_: Context) -> Result<Response> {
///     let data = vec![
///         User { id: 1, name: String::from("Mr.K") },
///         User { id: 2, name: String::from("ABC") },
///     ];
///     Resonse::OK(json(data)?)
/// }
/// ```
/// This is available to string types like `&str`, `String` because they implements `Serialize`, but `json()` doesn't performe any validation. If you'd like to make a json-format string into `JSON`, it will be easier to wrap it with `JSON` directly :\
/// ```no_run
/// let json_string: JSON = JSON("Hello, world!");
/// ```
#[allow(non_snake_case)]
pub fn json<S: Serialize>(data: S) -> Result<JSON> {
    Ok(JSON(serde_json::to_string(&data)?))
}

/// Type of raw json data in TCP stream.
#[derive(Debug, PartialEq, Clone)]
pub struct JSON(
    pub String
);
impl<'d> JSON {
    /// Try deserializing self into Rust struct implementing `serde::Deserialize` and return `Result</* that struct */>`.\
    /// \* To deserialize request body, ohkami users **should use** `Context::body<D>`. `to_struct` may be used to deserialize String read from, for example, static `.json` file.
    /// ```no_run
    /// let file = File::open("data.json");
    /// let data = JSON({
    ///     let mut buf = String::new();
    ///     file.read_to_string(&mut buf)?;
    ///     buf
    /// }).to_struct()?;
    /// ```
    pub fn to_struct<D: Deserialize<'d>>(&'d self) -> Result<D> {
        Ok(serde_json::from_str(&self.0)?)
    }

    pub(crate) fn content_length(&self) -> usize {
        self.0.len()
    }
}

*/

// impl<T: Serialize + for <'d> Deserialize<'d>> ResponseFormat for JSON<T> {
//     fn response_format(&self) -> &str {
//         self.ser().unwrap_or_else(|_| String::new()).as_str()
//     }
// }

/// Utility macro to create `JSON` value from some pair(s) of key-value(s).
/// ```no_run
/// let result = json!("ok": true);
/// ```
/// ```no_run
/// let res = json!("token": "abcxyz", "expires": "2022-01-01 00:00");
/// ```
#[macro_export]
macro_rules! json {
    ($key1:literal : $value1:expr $(, $key:literal : $value:expr)*) => {
        crate::response::body::Body::application_json(
            String::from("{")
            + &format!("\"{}\":{:?}", $key1, $value1)
            $( + &format!(",\"{}\":{:?}", $key, $value) )*
            + "}"
        )
    };
}
