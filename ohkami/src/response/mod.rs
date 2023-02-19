pub(crate) mod components;

use serde::Serialize;
use std::{ops::{ControlFlow, Try, FromResidual}, marker::PhantomData};
use self::{components::{
    status::Status,
    content_type::ContentType,
    header::ResponseHeaders,
    time::now,
}};

pub enum Response<T: Serialize> {
    Ok(OkResponse<T>),
    Err(ErrResponse),
}
struct OkResponse<T: Serialize>(
    String,
    PhantomData<fn() -> T>
);
pub struct ErrResponse(
    String
);


impl<T: Serialize> Try for Response<T> {
    type Residual = ErrResponse;
    type Output = OkResponse<T>;
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Err(err_res) => ControlFlow::Break(err_res),
            Self::Ok(ok_res) => ControlFlow::Continue(ok_res),
        }
    }
    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }
}

impl<T: Serialize> FromResidual<ErrResponse> for Response<T> {

}

impl<T: Serialize> Response<T> {
    #[inline] pub(crate) fn from(
        status: Status,
        content_type: ContentType,
        additional_headers: &ResponseHeaders,
        body: T,
    ) -> Self {
        match serde_json::to_string(&body) {
            Ok(body) => Self::Ok(OkResponse(format!(
"HTTP/1.1 {}
Connection: Keep-Alive
Keep-Alive: timeout=5
Content-Type: {}; charset=UTF-8
Content-Length: {}
Date: {}
{}
{}
",
                status.as_str(),
                content_type.as_str(),
                body.len(),
                now(),
                additional_headers,
                body
            ), PhantomData)),

            Err(_) => Self::Err(ErrResponse(format!(
"HTTP/1.1 {}
Connection: Keep-Alive
Keep-Alive: timeout=5
Content-Type: text/plain; charset=UTF-8
Content-Length: 19
Date: {}
{}
failed to serialize
",
                status.as_str(),
                now(),
                additional_headers
            )))
        }
    }
}
