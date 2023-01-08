use crate::prelude::Result;

pub trait Json<'j>: serde::Serialize + serde::Deserialize<'j> {
    fn ser(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
    fn de(string: &'j str) -> Result<Self> {
        Ok(serde_json::from_str(string)?)
    }
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