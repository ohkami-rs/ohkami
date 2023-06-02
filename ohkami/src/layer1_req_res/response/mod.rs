mod headers; pub(crate) use headers::ResponseHeaders;

use serde::Serialize;
use std::{
    ops::{Try, FromResidual, ControlFlow},
    marker::PhantomData,
    borrow::Cow,
};
use crate::{
    __dep__, __dep__::StreamWriter,
    layer0_lib::{Status, ContentType, AsStr, IntoCow},
};


pub type Response<T> = ::std::result::Result<OkResponse<T>, ErrorResponse>;
pub struct OkResponse<T: Serialize>(String, PhantomData<T>);
pub struct ErrorResponse {
    status_and_headers: String,
    content: Option<(ContentType, Cow<'static, str>)>
}

#[cfg(test)] fn __(
    make_result_1: fn() -> Result<usize, ErrorResponse>,
    make_result_2: fn() -> Result<usize, std::io::Error>,
) -> Response<()> {
    let _: usize = make_result_1()?;

    let _: usize = make_result_2()
        .map_err(|e| ErrorResponse{status_and_headers:todo!(), content:todo!()})?;

    todo!()
}

#[inline] pub(crate) async fn send<T: Serialize>(
    response: Response<T>,
    stream: &mut __dep__::TcpStream
) {
    if let Err(e) = stream.write_all(match response {
        Ok(ok_response)     => ok_response.0,
        Err(error_response) => {
            let ErrorResponse { status_and_headers, content } = error_response;
            let mut response = status_and_headers;
            if let Some((content_type, body)) = content {
                response.push('\r'); response.push('\n');
                response.push_str(content_type.as_str());
                response.push('\r'); response.push('\n');
                response.push('\r'); response.push('\n');
                response.push_str(&body);
            }
            response
        },
    }.as_bytes()).await {
        panic!("Failed to respond: {e}")
    }
}


impl<T: AsStr> OkResponse<T> {
    #[inline(always)] pub(crate) fn with_body_asstr(
        body: T,
        status: Status,
        headers: &ResponseHeaders,
    ) -> Self {
        let __status__ = status.as_str();
        let __headers__ = headers.as_str();
        let __body__ = body.as_str();
        let cl = __body__.len();

        OkResponse(format!(
"HTTP/1.1 {__status__}\r
{__headers__}\r
Content-Length: {cl}\r
\r
{__body__}"), PhantomData)
    }
}
impl<T: Serialize> OkResponse<T> {
    #[inline(always)] pub(crate) fn with_body(
        body: T,
        status: Status,
        headers: &ResponseHeaders,
    ) -> Self {
        let __status__ = status.as_str();
        let __headers__ = headers.as_str();
        let __body__ = serde_json::to_string(&body).expect("Failed to serialize");
        let cl = __body__.len();

        OkResponse(format!(
"HTTP/1.1 {__status__}\r
{__headers__}\r
Content-Length: {cl}\r
\r
{__body__}"), PhantomData)
    }
}
impl OkResponse<()> {
    #[inline(always)] pub(crate) fn without_body(
        status: Status,
        headers: &ResponseHeaders
    ) -> Self {
        let __status__ = status.as_str();
        let __headers__ = headers.as_str();

        OkResponse(format!(
"HTTP/1.1 {__status__}\r
{__headers__}"), PhantomData)
    }
}


impl ErrorResponse {
    #[inline(always)] pub(crate) fn new(
        status: Status,
        headers: &ResponseHeaders,
    ) -> ErrorResponse {
        let __status__ = status.as_str();
        let __headers__ = headers.as_str();

        ErrorResponse {
            content: None,
            status_and_headers: format!(
"HTTP/1.1 {__status__}\r
{__headers__}"),
        }
    }
}

impl ErrorResponse {
    #[inline(always)] pub fn text<Text: IntoCow<'static>>(mut self, text: Text) -> Self {
        self.content.replace((ContentType::Text, text.into_cow()));
        self
    }
    #[inline(always)] pub fn html<HTML: IntoCow<'static>>(mut self, html: HTML) -> Self {
        self.content.replace((ContentType::HTML, html.into_cow()));
        self
    }
    #[inline(always)] pub fn json<JSON: Serialize>(mut self, json: JSON) -> Self {
        let json = serde_json::to_string(&json).expect("Failed to serialize");
        self.content.replace((ContentType::JSON, Cow::Owned(json)));
        self
    }
}
