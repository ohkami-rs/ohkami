use crate::{IntoResponse, Request, Response};

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

    #[doc(hidden)]
    /// intent to be used by `format::Path` and by the assertion in `router::base::Router::finalize`
    fn n_params() -> usize {
        0
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

        if req.headers.content_type()?.starts_with(B::MIME_TYPE) {
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
