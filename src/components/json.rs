use serde::{Serialize, Deserialize};
use crate::{response::ResponseFormat, result::Result};


#[allow(non_snake_case)]
pub fn json<S: Serialize>(data: S) -> Result<JSON> {
    Ok(JSON(serde_json::to_string(&data)?))
}

#[derive(Debug)]
pub struct JSON(String);
impl<'d> JSON {
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


#[macro_export]
macro_rules! json {
    ($key1:literal : $value1:expr $(, $key:literal : $value:expr)*) => {
        json(
            String::from("{")
            + &format!("\"{}\":{:?}", $key1, $value1)
            $( + &format!(",\"{}\":{:?}", $key, $value) )*
            + "}"
        ).unwrap()
    };
}