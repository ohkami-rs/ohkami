#![allow(non_snake_case)]

use crate::{Response, Status};

#[cfg(all(debug_assertions, feature="openapi"))]
use crate::openapi;


/// A trait implemented to be a return value of a handler
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
///     ).howl("localhost:5050").await
/// }
/// ```
pub trait IntoResponse {
    fn into_response(self) -> Response;

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new(200, openapi::Response::when("OK"))
    }
}

impl IntoResponse for Response {
    #[inline] fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for Status {
    #[inline(always)] fn into_response(self) -> Response {
        Response::of(self)
    }
}

impl<T:IntoResponse, E:IntoResponse> IntoResponse for Result<T, E> {
    #[inline(always)] fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(e) => e.into_response(),
        }
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_responses() -> openapi::Responses {
        let mut res = E::openapi_responses();
        res.merge(T::openapi_responses());
        res
    }
}

impl IntoResponse for std::convert::Infallible {
    #[cold] #[inline(never)]
    fn into_response(self) -> Response {
        unsafe {std::hint::unreachable_unchecked()}
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::OK()
    }
}

macro_rules! text_response {
    ($( $t:ty )*) => {$(
        impl IntoResponse for $t {
            #[inline(always)]
            fn into_response(self: $t) -> Response {
                Response::OK().with_text(self)
            }

            #[cfg(all(debug_assertions, feature="openapi"))]
            fn openapi_responses() -> openapi::Responses {
                openapi::Responses::new(200, openapi::Response::when("OK")
                    .content("text/plain", openapi::Schema::string())
                )
            }
        }
    )*};
} text_response! {
    &'static str
    String
    std::borrow::Cow<'static, str>
}

#[cfg(feature="rt_worker")]
impl IntoResponse for worker::Error {
    fn into_response(self) -> Response {
        Response::InternalServerError().with_text(self.to_string())
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new(500, openapi::Response::when("Internal error in worker"))
    }
}
