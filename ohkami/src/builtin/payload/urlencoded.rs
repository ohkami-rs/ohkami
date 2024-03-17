use serde::{Serialize, Deserialize};
use ohkami_lib::serde_urlencoded;
use crate::typed::PayloadType;


pub struct URLEncoded;

impl PayloadType for URLEncoded {
    const CONTENT_TYPE: &'static str = "application/x-www-form-urlencoded";

    type Error = serde_urlencoded::Error;

    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, Self::Error> {
        let str = std::str::from_utf8(bytes).map_err(
            |e| serde::de::Error::custom(format!("input is not valid form-urlencoded: {e}"))
        )?;
        serde_urlencoded::from_str(str)
    }

    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, Self::Error> {
        serde_urlencoded::to_string(value).map(String::into_bytes)
    }
}
