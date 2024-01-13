#![allow(non_snake_case)]

use crate::{Response, layer0_lib::Status};


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

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::OK().text(self)
    }
}
impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::OK().text(self)
    }
}
impl IntoResponse for &'_ String {
    fn into_response(self) -> Response {
        Response::OK().text(self.clone())
    }
}
impl IntoResponse for std::borrow::Cow<'static, str> {
    fn into_response(self) -> Response {
        Response::OK().text(self)
    }
}
