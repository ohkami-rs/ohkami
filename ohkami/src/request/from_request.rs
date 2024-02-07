use std::borrow::Cow;
use crate::{Request};


pub enum FromRequestError {
    Static(&'static str),
    Owned(String),
} const _: () = {
    impl std::ops::Deref for FromRequestError {
        type Target = str;
        #[inline] fn deref(&self) -> &Self::Target {
            match self {
                Self::Owned (s) => &*s,
                Self::Static(s) => &*s,
            }
        }
    }
    impl From<std::borrow::Cow<'static, str>> for FromRequestError {
        #[inline] fn from(cow: std::borrow::Cow<'static, str>) -> Self {
            match cow {
                std::borrow::Cow::Borrowed(s) => Self::Static(s),
                std::borrow::Cow::Owned   (s) => Self::Owned (s),
            }
        }
    }
};

impl std::error::Error for FromRequestError {}
impl std::fmt::Debug for FromRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self)
    }
}
impl std::fmt::Display for FromRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self)
    }
}

/// Represents "retirieved from `Request`".
/// 
/// - `#[Query]`
/// - `#[Payload]`
/// 
/// implements this for a struct.
/// 
/// Of course, you can manually implement for any structs that can be extracted from request：
/// 
/// *example.rs*
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
    type Error: std::error::Error + 'static;
    fn from_request(req: &'req Request) -> Result<Self, Self::Error>;
}

/// Repredents "retrieved from path/query param".
pub trait FromParam<'p>: Sized {
    type Error: std::error::Error;
    /// `param` is percent-decoded：
    /// 
    /// - `Cow::Borrowed(&'p str)` by default
    /// - `Cow::Owned(String)` if it is decoded
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
        type Error = FromRequestError;
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            match param {
                Cow::Borrowed(s) => Ok(s),
                Cow::Owned(_)    => Err(FromRequestError::Static(
                    "Unexpected path params: Found percent decoded"
                )),
            }
        }
    }

    macro_rules! unsigned_integers {
        ($( $unsigned_int:ty ),*) => {
            $(
                impl<'p> FromParam<'p> for $unsigned_int {
                    type Error = FromRequestError;
                    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
                        let digit_bytes = param.as_bytes();
                        if digit_bytes.is_empty() {return Err(FromRequestError::Static("Unexpected path params: Expected a number nut found an empty string"))}
                        match digit_bytes[0] {
                            b'-' => Err(FromRequestError::Static("Unexpected path params: Expected non-negative number but found negetive one")),
                            b'0' => Err(FromRequestError::Static("Unexpected path params: Expected a number but it starts with '0'")),
                            _ => {
                                let mut value: $unsigned_int = 0;
                                for d in digit_bytes {
                                    match d {
                                        b'0'..=b'9' => value = value * 10 + (*d - b'0') as $unsigned_int,
                                        _ => return Err(FromRequestError::Static("Unexpected path params: Expected a number but it contains a non-digit charactor"))
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


