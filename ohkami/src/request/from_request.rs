use crate::{IntoResponse, Request};

#[cfg(feature = "openapi")]
use crate::openapi;

/// "Retirieved from a `Request`".
///
/// ### required
/// - `type Errpr: IntoResponse`
/// - `fn from_request(req: &Request) -> Option<Result<Self, Self::Error>>`
///
/// Of course, you can manually implement for your structs that can be extracted from a request：
///
/// <br>
///
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
///
/// struct IsGetRequest(bool);
///
/// impl ohkami::FromRequest<'_> for IsGetRequest {
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

    #[cfg(feature = "openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        openapi::Inbound::None
    }

    #[doc(hidden)]
    /// intent to be used by `claw::param::Path` and by the assertion in `router::base::Router::finalize`
    fn n_pathparams() -> usize {
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
                None => Some(Ok(None)),
                Some(fr) => Some(fr.map(Some)),
            }
        }

        #[cfg(feature = "openapi")]
        fn openapi_inbound() -> openapi::Inbound {
            FR::openapi_inbound()
        }
    }
};
#[cfg(feature = "rt_worker")]
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
