use serde::{Serialize, Deserialize};
use ohkami_lib::serde_utf8;
use crate::typed::PayloadType;


pub struct Text;
impl PayloadType for Text {
    const CONTENT_TYPE: &'static str = "text/plain";

    #[inline]
    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
        let str = std::str::from_utf8(bytes).map_err(
            |e| serde::de::Error::custom(format!("input is not UTF-8: {e}"))
        )?;
        serde_utf8::from_str(str)
    }

    #[inline(always)]
    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
        serde_utf8::to_string(value).map(String::into_bytes)
    }
}

/// This doesn't check the text is valid HTML.
pub struct HTML;
impl PayloadType for HTML {
    const CONTENT_TYPE: &'static str = "text/html";

    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
        serde_utf8::to_string(value).map(String::into_bytes)
    }

    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
        let str = std::str::from_utf8(bytes).map_err(
            |e| serde::de::Error::custom(format!("input is not UTF-8: {e}"))
        )?;
        serde_utf8::from_str(str)
    }
}
