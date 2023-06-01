mod headers; pub(crate) use headers::ResponseHeaders;

use serde::Serialize;
use std::{
    ops::{Try, FromResidual, ControlFlow},
    marker::PhantomData,
};
use crate::{
    __dep__, __dep__::StreamWriter,
    layer0_lib::{Status, ContentType, AsStr},
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


#[inline(always)] pub(crate) fn response_with_body_asstr<T: AsStr>(
    body: T,
    status: Status,
    headers: &ResponseHeaders,
) -> Response<T> {
    let __status__ = status.as_str();
    let __headers__ = headers.as_str();
    let __body__ = body.as_str();
    let cl = __body__.len();

    let response = format!(
"HTTP/1.1 {__status__}\r
{__headers__}\r
Content-Length: {cl}\r
\r
{__body__}");

    if status.is_error() {
        Err(ErrorResponse(response))
    } else {
        Ok(OkResponse(response, PhantomData))
    }
}

#[inline(always)] pub(crate) fn response_with_body<T: Serialize>(
    body: T,
    status: Status,
    headers: &ResponseHeaders,
) -> Response<T> {
    let __status__ = status.as_str();
    let __headers__ = headers.as_str();
    let __body__ = serde_json::to_string(&body).expect("Failed to serialize");
    let cl = __body__.len();

    let response = format!(
"HTTP/1.1 {__status__}\r
{__headers__}\r
Content-Length: {cl}\r
\r
{__body__}");

    if status.is_error() {
        Err(ErrorResponse(response))
    } else {
        Ok(OkResponse(response, PhantomData))
    }
}

#[inline(always)] pub(crate) fn response_without_body(
    status: Status,
    headers: &ResponseHeaders
) -> Response<()> {
    let __status__ = status.as_str();
    let __headers__ = headers.as_str();

    let response = format!(
"HTTP/1.1 {__status__}\r
{__headers__}");

    if status.is_error() {
        Err(ErrorResponse(response))
    } else {
        Ok(OkResponse(response, PhantomData))
    }
}
