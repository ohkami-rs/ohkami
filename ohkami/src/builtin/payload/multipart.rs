use serde::Deserialize;
use ohkami_lib::serde_multipart;
use crate::typed::PayloadType;


pub struct Multipart;
impl PayloadType for Multipart {
    const CONTENT_TYPE: &'static [&'static str] = &[
        "multipart/form-data"
    ];

    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
        serde_multipart::from_bytes(input)
    }
    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
        Err(crate::serde::de::Error::custom(
            "ohkami's builtin `Multipart` payload type doesn't support response use"
        ))
    }
}
