use std::borrow::Cow;
use crate::{util::ErrorMessage, IntoResponse, Request, Response};

#[cfg(feature="openapi")]
use crate::openapi;


/// "Retirieved from a `Request`".
/// 
/// ### required
/// - `type Errpr`
/// - `fn from_request`
/// 
/// Of course, you can manually implement for your structs that can be extracted from a request：
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// struct IsGETRequest(bool);
/// 
/// impl ohkami::FromRequest<'_> for IsGETRequest {
///     type Error = std::convert::Infallible;
///     fn from_request(req: &Request) -> Option<Result<Self, Self::Error>> {
///         Some(Ok(Self(
///             req.method.isGET()
///         )))
///     }
/// }
/// ```
/// 
/// <br>
/// 
/// ### Note
/// 
/// *MUST NOT impl both `FromRequest` and `FromParam`*.
pub trait FromRequest<'req>: Sized {
    /// If this extraction never fails, `std::convert::Infallible` is recomended.
    type Error: IntoResponse;
    
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>>;

    #[cfg(feature="openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::None
    }
}
const _: () = {
    impl<'req> FromRequest<'req> for &'req Request {
        type Error = std::convert::Infallible;
        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            Some(Ok(req))
        }
    }
    impl<'req, FR: FromRequest<'req>> FromRequest<'req> for Option<FR> {
        type Error = FR::Error;

        #[inline]
        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            match FR::from_request(req) {
                None     => Some(Ok(None)),
                Some(fr) => Some(fr.map(Some))
            }
        }

        #[cfg(feature="openapi")]
        fn openapi_inbound() -> openapi::Inbound {
            FR::openapi_inbound()
        }
    }
};
#[cfg(feature="rt_worker")]
const _: () = {
    impl<'req> FromRequest<'req> for &'req ::worker::Env {
        type Error = std::convert::Infallible;
        #[inline(always)]
        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            Some(Ok(req.context.env()))
        }
    }
    impl<'req> FromRequest<'req> for &'req ::worker::Context {
        type Error = std::convert::Infallible;
        #[inline(always)]
        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            Some(Ok(req.context.worker()))
        }
    }
};

/// "Retrieved from a path param".
/// 
/// ### required
/// - `type Errpr`
/// - `fn from_param`
/// 
/// NOTE: *MUST NOT impl both `FromRequest` and `FromParam`*.
pub trait FromParam<'p>: Sized {
    /// If this extraction never fails, `std::convert::Infallible` is recomended.
    type Error: IntoResponse;

    /// `param` is already percent-decoded：
    /// 
    /// - `Cow::Borrowed(&'p str)` if not encoded in request
    /// - `Cow::Owned(String)` if encoded and ohkami has decoded
    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error>;

    #[inline(always)]
    fn from_raw_param(raw_param: &'p [u8]) -> Result<Self, Response> {
        Self::from_param(
            ohkami_lib::percent_decode_utf8(raw_param)
                .map_err(|_e| {
                    #[cfg(debug_assertions)] crate::warning!(
                        "Failed to decode percent encoded param `{}`: {_e}",
                        raw_param.escape_ascii()
                    );
                    Response::InternalServerError()
                })?
        ).map_err(IntoResponse::into_response)
    }

    #[cfg(feature="openapi")]
    fn openapi_param() -> openapi::Parameter {
        openapi::Parameter::in_path(openapi::string())
    }
}
const _: () = {
    impl<'p> FromParam<'p> for String {
        type Error = std::convert::Infallible;

        #[inline(always)]
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            Ok(match param {
                Cow::Owned(s)    => s,
                Cow::Borrowed(s) => s.into()
            })
        }
    }
    impl<'p> FromParam<'p> for Cow<'p, str> {
        type Error = std::convert::Infallible;

        #[inline(always)]
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            Ok(param)
        }
    }
    impl<'p> FromParam<'p> for &'p str {
        type Error = ErrorMessage;

        #[inline(always)]
        fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
            match param {
                Cow::Borrowed(s) => Ok(s),
                Cow::Owned(_) => Err({
                    #[cold] #[inline(never)]
                    fn unexpected(param: &str) -> ErrorMessage {                        
                        crate::warning!("\
                            `&str` can't handle percent encoded parameters. \
                            Use `Cow<'_, str>` or `String` to handle them. \
                        ");
                        ErrorMessage(format!(    
                            "Unexpected path params `{param}`: percent encoded"
                        ))
                    } unexpected(&param)
                }),
            }
        }
    }

    macro_rules! unsigned_integers {
        ($( $unsigned_int:ty ),*) => {
            $(
                impl<'p> FromParam<'p> for $unsigned_int {
                    type Error = ErrorMessage;

                    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
                        ::byte_reader::Reader::new(param.as_bytes())
                            .read_uint()
                            .map(|i| Self::try_from(i).ok())
                            .flatten()
                            .ok_or_else(|| ErrorMessage(format!("Unexpected path param")))
                    }

                    #[cfg(feature="openapi")]
                    fn openapi_param() -> openapi::Parameter {
                        openapi::Parameter::in_path(openapi::integer())
                    }
                }
            )*
        };
    } unsigned_integers! { u8, u16, u32, u64, usize }

    macro_rules! signed_integers {
        ($( $signed_int:ty ),*) => {
            $(
                impl<'p> FromParam<'p> for $signed_int {
                    type Error = ErrorMessage;

                    fn from_param(param: Cow<'p, str>) -> Result<Self, Self::Error> {
                        ::byte_reader::Reader::new(param.as_bytes())
                            .read_int()
                            .map(|i| Self::try_from(i).ok())
                            .flatten()
                            .ok_or_else(|| ErrorMessage(format!("Unexpected path param")))
                    }

                    #[cfg(feature="openapi")]
                    fn openapi_param() -> openapi::Parameter {
                        openapi::Parameter::in_path(openapi::integer())
                    }
                }
            )*
        };
    } signed_integers! { i8, i16, i32, i64, isize }
};

pub trait FromBody<'req>: Sized {
    /// e.g. `application/json` `text/html`
    const MIME_TYPE: &'static str;

    fn from_body(body: &'req [u8]) -> Result<Self, impl std::fmt::Display>;

    #[cfg(feature="openapi")]
    fn openapi_requestbody() -> impl Into<openapi::schema::SchemaRef>;
}
impl<'req, B: FromBody<'req>> FromRequest<'req> for B {
    type Error = Response;
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        #[cold] #[inline(never)]
        fn reject(msg: impl std::fmt::Display) -> Response {
            Response::BadRequest().with_text(msg.to_string())
        }

        if req.headers.ContentType()?.starts_with(B::MIME_TYPE) {
            Some(B::from_body(req.payload()?).map_err(reject))
        } else {
            None
        }
    }

    #[cfg(feature="openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::Body(openapi::RequestBody::of(
            B::MIME_TYPE, B::openapi_requestbody()
        ))
    }
}
