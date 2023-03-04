pub(crate) mod components;

use async_std::{net::TcpStream, io::WriteExt};
use serde::Serialize;
use std::{ops::{ControlFlow, Try, FromResidual}, marker::PhantomData};
use self::{components::{
    status::Status,
    content_type::ContentType,
    header::ResponseHeaders,
    time::now, body::Text,
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
    #[inline] fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Err(err_res) => ControlFlow::Break(err_res),
            Self::Ok(ok_res) => ControlFlow::Continue(ok_res),
        }
    }
    #[inline] fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }
}
impl<T: Serialize> FromResidual<ErrResponse> for Response<T> {
    #[inline] fn from_residual(residual: ErrResponse) -> Self {
        Self::Err(residual)
    }
}


impl<T: Serialize> Response<T> {
    #[inline] pub(crate) async fn send(self, stream: &mut TcpStream) {
        if let Err(e) = stream.write_all(
            match self {
                Self::Ok(o)  => o.0,
                Self::Err(e) => e.0,
            }.as_bytes()
        ).await {
            tracing::error!("{e}"); panic!()
        }
    }
}

impl<T: Serialize> Response<T> {
    #[inline] pub(crate) fn with_body(
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
                additional_headers.as_str(),
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
                additional_headers.as_str()
            )))
        }
    }

    #[inline]
    pub(crate) fn error<Msg: Text>(
        status: Status,
        additional_headers: &ResponseHeaders,
        message: Msg,
    ) -> Self {
        let message = message.as_str();
        Self::Err(ErrResponse(format!(
"HTTP/1.1 {}
Connection: Keep-Alive
Keep-Alive: timeout=5
Content-Type: text/plain; charset=UTF-8
Content-Length: {}
Date: {}
{}
{}
",
            status.as_str(),
            message.len(),
            now(),
            additional_headers.as_str(),
            message
        )))
    }
}

impl Response<()> {
    #[inline] pub(crate) fn no_content(
        additional_headers: &ResponseHeaders
    ) -> Self {
        Self::Ok(OkResponse(format!(
"HTTP/1.1 204 No Content
Connection: Keep-Alive
Keep-Alive: timeout=5
Date: {}
{}
",
            now(),
            additional_headers.as_str()
        ), PhantomData))
    }
}
