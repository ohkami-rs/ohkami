mod headers; pub(crate) use headers::ResponseHeaders;

use serde::Serialize;
use std::{
    ops::{Try, FromResidual, ControlFlow},
    marker::PhantomData,
};
use crate::{
    __dep__, __dep__::StreamWriter,
    layer0_lib::{Status, ContentType},
};


pub type Response<T> = ::std::result::Result<OkResponse<T>, ErrorResponse>;
pub struct OkResponse<T: Serialize>(String, PhantomData<T>);
pub struct ErrorResponse(String);

#[cfg(test)] fn __(
    make_result_1: fn() -> Result<usize, ErrorResponse>,
    make_result_2: fn() -> Result<usize, std::io::Error>,
) -> Response<()> {
    let _: usize = make_result_1()?;

    let _: usize = make_result_2()
        .map_err(|e| ErrorResponse(e.to_string()))?;

    todo!()
}

#[inline] pub(crate) async fn send<T: Serialize>(
    response: Response<T>,
    stream: &mut __dep__::TcpStream
) {
    if let Err(e) = stream.write_all(match response {
        Ok(ok_response)     => ok_response.0,
        Err(error_response) => error_response.0,
    }.as_bytes()).await {
        panic!("Failed to respond: {e}")
    }
}

#[inline(always)] pub(crate) fn with_body<T: Serialize>(
    status: Status,
    headers: &ResponseHeaders,
    body: T,
) -> Response<T> {
    let __status__ = status.as_str();
    let __headers__ = headers.as_str();
    let __body__ = serde_json::to_string(&body).expect("Failed to serialize");

    let response = format!(
"HTTP/1.1 {__status__}\r
{__headers__}\r
\r
{__body__}");

    if status.is_error() {
        Err(ErrorResponse(response))
    } else {
        Ok(OkResponse(response, PhantomData))
    }
}
