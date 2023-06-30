mod headers; pub(crate) use headers::ResponseHeaders;

use serde::Serialize;
use std::{
    borrow::Cow,
    marker::PhantomData,
    ops::FromResidual,
    convert::Infallible,
};
use crate::{
    __dep__, __dep__::AsyncWriter,
    layer0_lib::{Status, ContentType, AsStr, IntoCow},
};


pub enum Response<T: Serialize = ()> {
    Ok(String, PhantomData<fn()->T>),
    Err(ErrResponse),
}
pub struct ErrResponse {
    status_and_headers: String,
    content: Option<(ContentType, Cow<'static, str>)>,
}
const _: () = {
    impl<T: Serialize> FromResidual<Result<Infallible, ErrResponse>> for Response<T> {
        fn from_residual(result: Result<Infallible, ErrResponse>) -> Self {
            Response::Err(result.expect_err("Can't convert Result::Ok(_) into Response"))
        }
    }
};

#[cfg(test)] fn __(
    make_result_1: fn() -> Result<usize, ErrResponse>,
    make_result_2: fn() -> Result<usize, std::io::Error>,
) -> Response<()> {
    let _: usize = make_result_1()?;

    let _: usize = make_result_2()
        .map_err(|_| ErrResponse { status_and_headers: String::new(), content: None })?;

    todo!()
}

impl ErrResponse {
    #[inline(always)] pub(crate) fn new(
        status: Status,
        headers: &ResponseHeaders,
    ) -> Self {
        let __status__ = status.as_str();
        let __headers__ = headers.to_string();

        Self { status_and_headers: format!(
            "HTTP/1.1 {__status__}\r\n\
            {__headers__}" //: a series of lines ending with "\r\n"
        ), content: None }
    }
    #[inline] pub(crate) fn to_string(self) -> String {
        match self {
            ErrResponse { status_and_headers: mut res, content:None } => {
                res.push('\r');
                res.push('\n');
                res
            }
            ErrResponse { status_and_headers: mut res, content: Some((content_type, body)) } => {
                let __content_type__ = content_type.as_str();
                let __content_length__ = body.len();

                res.push_str(&format!("\
                    Content-Type: {__content_type__}\r\n\
                    Content-Length: {__content_length__}\r\n\
                    \r\n\
                    {body}\
                "));

                res
            }
        }
    }

    #[inline(always)] #[allow(non_snake_case)] pub fn Text<Text: IntoCow<'static>>(mut self, text: Text) -> Self {
        self.content.replace((ContentType::Text, text.into_cow()));
        self
    }
    #[inline(always)] #[allow(non_snake_case)] pub fn HTML<HTML: IntoCow<'static>>(mut self, html: HTML) -> Self {
        self.content.replace((ContentType::HTML, html.into_cow()));
        self
    }
    #[inline(always)] #[allow(non_snake_case)] pub fn JSON<JSON: Serialize>(mut self, json: JSON) -> Self {
        let json = serde_json::to_string(&json).expect("Failed to serialize");
        self.content.replace((ContentType::JSON, Cow::Owned(json)));
        self
    }
}

impl<T: Serialize> Response<T> {
    #[inline] pub(crate) fn to_string(self) -> String {
        match self {
            Self::Ok(res, _)   => res,
            Self::Err(err_res) => err_res.to_string(),
        }
    }
    #[inline(always)] pub(crate) async fn send(self, stream: &mut __dep__::TcpStream) {
        if let Err(e) = stream.write_all(self.to_string().as_bytes()).await {
            panic!("Failed to respond: {e}")
        }
    }
}

impl<T: AsStr> Response<T> {
    #[inline(always)] pub(crate) fn ok_with_body_asstr(
        body: T,
        status: Status,
        content_type: ContentType,
        headers: &ResponseHeaders,
    ) -> Self {
        let __status__ = status.as_str();
        let __headers__ = headers.to_string();
        let __content_type__ = content_type.as_str();
        let __body__ = body.as_str();
        let __content_length__ = __body__.len();

        Self::Ok(format!(
            "HTTP/1.1 {__status__}\r\n\
            Content-Type: {__content_type__}\r\n\
            Content-Length: {__content_length__}\r\n\
            {__headers__}\
            \r\n\
            {__body__}"
        ), PhantomData)
    }
}
impl<T: Serialize> Response<T> {
    #[inline(always)] pub(crate) fn ok_with_body_json(
        body: T,
        status: Status,
        headers: &ResponseHeaders,
    ) -> Self {
        let __status__ = status.as_str();
        let __headers__ = headers.to_string();
        let __body__ = serde_json::to_string(&body).expect("Failed to serialize");
        let __content_length__ = __body__.len();

        Self::Ok(format!(
            "HTTP/1.1 {__status__}\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {__content_length__}\r\n\
            {__headers__}\
            \r\n\
            {__body__}"
        ), PhantomData)
    }
}
impl Response {
    #[inline(always)] pub(crate) fn ok_without_body(
        status: Status,
        headers: &ResponseHeaders,
    ) -> Self {
        let __status__ = status.as_str();
        let __headers__ = headers.to_string();

        Self::Ok(format!(
            "HTTP/1.1 {__status__}\r\n\
            {__headers__}\
            \r\n"
        ), PhantomData)
    }

    #[inline(always)] pub(crate) fn redirect(
        location: impl AsStr,
        status: Status,
        headers: &ResponseHeaders,
    ) -> Self {
        let __location__ = location.as_str();
        let __status__ = status.as_str();
        let __headers__ = headers.to_string();

        Self::Ok(format!(
            "HTTP/1.1 {__status__}\r\n\
            Location: {__location__}\r\n\
            {__headers__}\
            \r\n"
        ), PhantomData)
    }
}
