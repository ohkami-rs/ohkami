use serde::{Serialize, Deserialize};
use crate::{response::ResponseFormat, result::Result};


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
#[derive(Debug, PartialEq)]
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

impl ResponseFormat for JSON {
    fn response_format(&self) -> &str {
        self.0.as_str()
    }
}


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
        JSON(
            String::from("{")
            + &format!("\"{}\":{:?}", $key1, $value1)
            $( + &format!(",\"{}\":{:?}", $key, $value) )*
            + "}"
        )
    };
}