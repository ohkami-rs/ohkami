use ::byte_reader::Reader;
use std::pin::Pin;
use super::Error;
use crate::Slice;


pub(crate) struct MultipartDesrializer<'de> {
    r:        Reader<'de>,
    boundary: Slice,
}

impl<'de> MultipartDesrializer<'de> {
    pub(crate) fn new(input: &'de [u8]) -> Result<Self, Error> {
        let mut r = Reader::new(input);

        r.consume("--").ok_or_else(Error::ExpectedValidBoundary)?;
        // SAFETY:
        //    What `boundary` refers to is `input`, that keeps alive
        //    for 'de, the same lifetime as `Self`
        let boundary = unsafe {Slice::from_bytes(
            r.read_while(|b| b != &b'\r')
        )};
        r.consume("\r\n").ok_or_else(Error::MissingCRLF)?;

        Ok(Self { r, boundary })
    }
    pub(crate) fn remaining(&self) -> &[u8] {
        self.r.remaining()
    }
}

impl<'de> serde::de::Deserializer<'de> for &mut MultipartDesrializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        todo!()
    }
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where V: serde::de::Visitor<'de> {
        
    }
}
