use std::borrow::Cow;
use crate::{Request};


/// Implmented for structs you put
/// 
/// - `#[Queries]`
/// - `#[Headers]`
/// - `#[Payload(JSON)]`
/// - `#[Payload(Form)]`
/// - `#[Payload(URLEncoded)]`
/// 
/// <br/>
/// 
/// And, you can manually implement for any structs that can be extracted from request：
/// 
/// ```ignore
/// struct HasPayload(bool);
/// 
/// impl FromRequest for HasPayload {
///     fn parse(req: &Request) -> Result<Self, Cow> {
///         Ok(Self(
///             req.payload.is_some()
///         ))
///     }
/// }
/// ```
pub trait FromRequest: Sized {
    fn parse(req: &Request) -> Result<Self, Cow<'static, str>>;
}

pub trait PathParam: Sized {
    fn parse(bytes: &[u8]) -> Result<Self, Cow<'static, str>>;
} const _: () = {
    impl PathParam for &str {
        fn parse(bytes: &[u8]) -> Result<Self, Cow<'static, str>> {
            // SAFETY:
            // - This str refers to `buffer` in `Request`
            // - And, this str is visible to user **ONLY IN** the handler
            //   that is handling the request
            Ok(unsafe {std::mem::transmute(
                std::str::from_utf8(bytes)
                    .map_err(move |e| Cow::Owned(e.to_string()))?
            )})
        }
    }

    impl PathParam for String {
        fn parse(bytes: &[u8]) -> Result<Self, Cow<'static, str>> {
            Ok(String::from(
                std::str::from_utf8(bytes)
                    .map_err(move |e| Cow::Owned(e.to_string()))?
            ))
        }
    }

    macro_rules! unsigned_integers {
        ($( $unsigned_number_type:ty ),*) => {
            $(
                impl PathParam for $unsigned_number_type {
                    fn parse(bytes: &[u8]) -> Result<Self, Cow<'static, str>> {
                        if bytes.is_empty() {return Err(Cow::Borrowed("Expected a number nut found an empty string"))}
                        match bytes[0] {
                            b'-' => Err(Cow::Borrowed("Expected non-negative number but found negetive one")),
                            b'0' => Err(Cow::Borrowed("Expected a number but it starts with '0'")),
                            _ => {
                                let mut value: $unsigned_number_type = 0;
                                for d in bytes {
                                    match d {
                                        0..=9 => value = value * 10 + *d as $unsigned_number_type,
                                        _ => return Err(Cow::Borrowed("Expected a number but it contains a non-digit charactor"))
                                    }
                                }
                                Ok(value)
                            }
                        }
                    }
                }
            )*
        };
    } unsigned_integers! { u8, u16, u32, u64, u128, usize }
};


