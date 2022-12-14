use serde::{Serialize, Deserialize};
use crate::{response::ResponseFormat, result::Result};


#[derive(Debug)]
pub struct JSON(String);
impl<'d> JSON {
    pub fn from_struct<S: Serialize>(value: &S) -> Result<Self> {
        Ok(Self(serde_json::to_string(value)?))
    }
    pub fn to_struct<D: Deserialize<'d>>(&'d self) -> Result<D> {
        Ok(serde_json::from_str(&self.0)?)
    }
    pub fn from_str<Str: ToString>(str: Str) -> Self {
        Self(str.to_string())
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


#[macro_export]
macro_rules! json {
    ($key1:literal : $value1:expr $(, $key:literal : $value:expr)*) => {
        JSON::from_str(
            String::from("{")
            + &format!("\"{}\":{:?}", $key1, $value1)
            $( + &format!(",\"{}\":{:?}", $key, $value) )*
            + "}"
        )
    };
}