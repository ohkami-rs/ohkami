#![allow(non_snake_case)]

use crate::{Response, Status};

#[cfg(feature = "openapi")]
use crate::openapi;

/// A trait implemented to be a returned value of a handler
///
/// <br>
///
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
///
/// struct MyResponse {
///     message: String,
/// }
/// impl IntoResponse for MyResponse {
///     fn into_response(self) -> Response {
///         Response::OK().with_text(self.message)
///     }
/// }
///
/// async fn handler() -> MyResponse {
///     MyResponse {
///         message: String::from("Hello!")
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new(
///         "/".GET(handler)
///     ).run("localhost:5050").await
/// }
/// ```
pub trait IntoResponse {
    fn into_response(self) -> Response;

    #[cfg(feature = "openapi")]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new([])
    }
}

impl IntoResponse for Response {
    #[inline]
    fn into_response(self) -> Response {
        self
    }

    #[cfg(feature = "openapi")]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new([])
    }
}

impl IntoResponse for Status {
    #[inline(always)]
    fn into_response(self) -> Response {
        Response::new(self)
    }

    #[cfg(feature = "openapi")]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new([])
    }
}

impl<T: IntoResponse, E: IntoResponse> IntoResponse for Result<T, E> {
    #[inline(always)]
    fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(e) => e.into_response(),
        }
    }

    #[cfg(feature = "openapi")]
    fn openapi_responses() -> openapi::Responses {
        let mut res = E::openapi_responses();
        res.merge(T::openapi_responses());
        res
    }
}

impl IntoResponse for std::convert::Infallible {
    #[cold]
    #[inline(never)]
    fn into_response(self) -> Response {
        unsafe { std::hint::unreachable_unchecked() }
    }

    #[cfg(feature = "openapi")]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new([])
    }
}

#[cfg(feature = "rt_worker")]
impl IntoResponse for worker::Error {
    fn into_response(self) -> Response {
        Response::InternalServerError().with_text(self.to_string())
    }

    #[cfg(feature = "openapi")]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new([(500, openapi::Response::when("Internal error in worker"))])
    }
}
