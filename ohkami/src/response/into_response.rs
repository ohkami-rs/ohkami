#![allow(non_snake_case)]

use crate::{Response, Status};


/// Represents "can be handlers' return value".
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
///         Response::with(Status::OK)
///             .text(self.message)
///     }
/// }
/// 
/// async fn handler() -> MyResponse {
///     MyResponse {
///         message: "Hello!".to_string()
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
}

impl IntoResponse for Response {
    #[inline] fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for Status {
    #[inline(always)] fn into_response(self) -> Response {
        Response::with(self)
    }
}

impl<T:IntoResponse, E:IntoResponse> IntoResponse for Result<T, E> {
    #[inline(always)] fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for std::convert::Infallible {
    #[cold]
    fn into_response(self) -> Response {
        unsafe {std::hint::unreachable_unchecked()}
    }
}
