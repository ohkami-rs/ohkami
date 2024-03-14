use serde::{Serialize, Deserialize};
use crate::typed::{Payload, PayloadType};


pub struct Text;
impl PayloadType for Text {
    const CONTENT_TYPE: &'static str = "text/plain";

    type Error = std::str::Utf8Error;

    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, Self::Error> {
        

        todo!()
    }

    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, Self::Error> {
        
    }
}
