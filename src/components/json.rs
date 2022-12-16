use serde::{Serialize, Deserialize};
use crate::{response::ResponseFormat, result::Result};


#[allow(non_snake_case)]
pub fn JSON<S: Serialize>(data: S) -> Result<Json> {
    Ok(Json(serde_json::to_string(&data)?))
}

#[derive(Debug)]
pub struct Json(String);
impl<'d> Json {
    pub(crate) fn to_struct<D: Deserialize<'d>>(&'d self) -> Result<D> {
        Ok(serde_json::from_str(&self.0)?)
    }
    pub(crate) fn content_length(&self) -> usize {
        self.0.len()
    }
}

impl ResponseFormat for Json {
    fn response_format(&self) -> &str {
        self.0.as_str()
    }
}


#[macro_export]
macro_rules! json {
    ($key1:literal : $value1:expr $(, $key:literal : $value:expr)*) => {
        JSON(
            String::from("{")
            + &format!("\"{}\":{:?}", $key1, $value1)
            $( + &format!(",\"{}\":{:?}", $key, $value) )*
            + "}"
        ).unwrap()
    };
}