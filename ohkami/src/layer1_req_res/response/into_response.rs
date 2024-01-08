#![allow(non_snake_case)]

use crate::{Response, layer0_lib::Status};
use super::ResponseHeaders;


pub trait IntoResponse {
    fn into_response(self) -> Response;
}


impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}
impl crate::IntoResponse for Status {
    fn into_response(self) -> crate::Response {
        crate::Response {
            status:  self,
            headers: ResponseHeaders::new(),
            content: None,
        }
    }
}
impl<'req, T:IntoResponse, E:IntoResponse> IntoResponse for Result<T, E> {
    fn into_response(self) -> Response {
        match self {
            Ok(ok) => ok.into_response(),
            Err(e) => e.into_response(),
        }
    }
}
