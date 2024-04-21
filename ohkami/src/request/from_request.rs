use std::borrow::Cow;
use crate::{IntoResponse, Request, Response};


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

    impl IntoResponse for FromRequestError {
        fn into_response(self) -> Response {
            Response::InternalServerError().text(match self {
                Self::Owned(s)  => Cow::Owned(s),
                Self::Static(s) => Cow::Borrowed(s),
            })
        }
    }
};

/// "Retirieved from a `Request`".
/// 
/// - `#[Query]`
/// - `#[Payload]`
/// 
/// derives `FromRequest` impl for a struct.
/// 
/// Of course, you can manually implement for your structs that can be extracted from a request：
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// struct IsGETRequest(bool);
/// 
/// impl ohkami::FromRequest<'_> for IsGETRequest {
///     type Error = std::convert::Infallible;
///     fn from_request(req: &Request) -> Result<Self, Self::Error> {
///         Ok(Self(
///             req.method().isGET()
///         ))
///     }
/// }
/// ```
/// ---
/// 
/// <br>
/// 
/// NOTE: *Cannot impl both `FromRequest` and `FromParam`*.
pub trait FromRequest<'req>: Sized {
    /// If this extraction never fails, `std::convert::Infallible` is recomended.
    type Error: IntoResponse;
    
    fn from_request(req: &'req Request) -> Result<Self, Self::Error>;
} const _: () = {
    impl<'req> FromRequest<'req> for &'req Request {
        type Error = std::convert::Infallible;
        fn from_request(req: &'req Request) -> Result<Self, Self::Error> {
            Ok(req)
        }
    }
};

/// "Retrieved from a path/query param".
/// 
/// NOTE: *Cannot impl both `FromRequest` and `FromParam`*.
pub trait FromParam<'p>: Sized {
    /// If this extraction never fails, `std::convert::Infallible` is recomended.
    type Error: IntoResponse;

    /// `param` is percent-decoded：
    /// 
    /// - `Cow::Borrowed(&'p str)` by default
    /// - `Cow::Owned(String)` if ohkami has decoded it
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
            #[cold] fn unexpectedly_percent_encoded() -> FromRequestError {
                eprintln!("\
                    \n\
                    =========\n\
                    [WARNING] \
                    `&str` can't handle percent encoded parameters. \
                    Use `Cow<'_, str>` (or `String` instead) \
                    to handle them.\n\
                    =========\n\
                ");
                FromRequestError::Owned(format!(    
                    "Unexpected path params: percent encoded"
                ))
            }

            match param {
                Cow::Borrowed(s) => Ok(s),
                Cow::Owned(_)    => Err(unexpectedly_percent_encoded()),
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
