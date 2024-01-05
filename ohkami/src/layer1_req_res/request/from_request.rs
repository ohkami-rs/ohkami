use std::borrow::Cow;
use crate::{Request};


/// Represents "this is retirieved from `Request`".
/// 
/// - `#[Query]`
/// - `#[Payload(JSON)]`
/// - `#[Payload(URLEncoded)]`
/// - `#[Payload(Form)]`
/// 
/// implement this by default.
/// 
/// <br/>
/// 
/// Of course, you can manually implement for any structs that can be extracted from request：
/// 
/// ```
/// use ohkami::prelude::*;
/// 
/// struct HasPayload(bool);
/// 
/// impl ohkami::FromRequest<'_> for HasPayload {
///     type Error = std::convert::Infallible;
///     fn from_request(req: &Request) -> Result<Self, Self::Error> {
///         Ok(Self(
///             req.payload().is_some()
///         ))
///     }
/// }
/// ```
pub trait FromRequest<'req>: Sized {
    type Error: std::fmt::Display + 'static;
    fn from_request(req: &'req Request) -> Result<Self, Self::Error>;
}

pub trait FromParam<'p>: Sized {
    type Error: std::fmt::Display;
    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error>;
} const _: () = {
    impl<'p> FromParam<'p> for String {
        type Error = std::convert::Infallible;
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            Ok(param.to_string())
        }
    }
    impl<'p> FromParam<'p> for Cow<'p, str> {
        type Error = std::convert::Infallible;
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            Ok(param)
        }
    }
    impl<'p> FromParam<'p> for &'p str {
        type Error = &'static str;
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            match param {
                Cow::Borrowed(s) => Ok(s),
                Cow::Owned(_)    => Err("Found percent decoded `String`, can't convert into `&str` in `from_param`"),
            }
        }
    }

    macro_rules! unsigned_integers {
        ($( $unsigned_int:ty ),*) => {
            $(
                impl<'p> FromParam<'p> for $unsigned_int {
                    type Error = Cow<'static, str>;
                    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
                        let digit_bytes = param.as_bytes();
                        if digit_bytes.is_empty() {return Err(Cow::Borrowed("Expected a number nut found an empty string"))}
                        match digit_bytes[0] {
                            b'-' => Err(Cow::Borrowed("Expected non-negative number but found negetive one")),
                            b'0' => Err(Cow::Borrowed("Expected a number but it starts with '0'")),
                            _ => {
                                let mut value: $unsigned_int = 0;
                                for d in digit_bytes {
                                    match d {
                                        b'0'..=b'9' => value = value * 10 + (*d - b'0') as $unsigned_int,
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


