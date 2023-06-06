use std::borrow::Cow;
use crate::{Error, Request};


pub trait Queries: Sized {
    fn parse(req: &Request) -> Result<Self, Error>;
}

pub trait Payload: Sized {
    fn parse(req: &Request) -> Result<Self, Error>;
}

pub trait PathParam: Sized {
    fn parse(bytes: &[u8]) -> Result<Self, Error>;
} const _: () = {
    impl PathParam for &str {
        fn parse(bytes: &[u8]) -> Result<Self, Error> {
            // SAFETY:
            // - This str refers to `buffer` in `Request`
            // - And, this str is visible to user **ONLY IN** the handler
            //   that is handling the request
            Ok(unsafe {std::mem::transmute(
                std::str::from_utf8(bytes)
                    .map_err(move |e| Error::Parse(Cow::Owned(e.to_string())))?
            )})
        }
    }

    impl PathParam for String {
        fn parse(bytes: &[u8]) -> Result<Self, Error> {
            Ok(String::from(
                std::str::from_utf8(bytes)
                    .map_err(move |e| Error::Parse(Cow::Owned(e.to_string())))?
            ))
        }
    }

    macro_rules! unsigned_numbers {
        ($( $unsigned_number_type:ty ),*) => {
            $(
                impl PathParam for $unsigned_number_type {
                    fn parse(bytes: &[u8]) -> Result<Self, Error> {
                        if bytes.is_empty() {return Err(Error::Parse(Cow::Borrowed("Expected a number nut found an empty string")))}
                        match bytes[0] {
                            b'-' => Err(Error::Parse(Cow::Borrowed("Expected non-negative number but found negetive one"))),
                            b'0' => Err(Error::Parse(Cow::Borrowed("Expected a number but it starts with '0'"))),
                            _ => {
                                let mut value: $unsigned_number_type = 0;
                                for d in bytes {
                                    match d {
                                        0..=9 => value = value * 10 + *d as $unsigned_number_type,
                                        _ => return Err(Error::Parse(Cow::Borrowed("Expected a number but it contains a non-digit charactor")))
                                    }
                                }
                                Ok(value)
                            }
                        }
                    }
                }
            )*
        };
    } unsigned_numbers! { u8, u16, u32, u64, u128, usize }
};


