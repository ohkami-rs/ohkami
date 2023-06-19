mod headers; pub(crate) use headers::ResponseHeaders;

use serde::Serialize;
use std::{
    borrow::Cow,
    marker::PhantomData,
    ops::FromResidual,
    convert::Infallible,
};
use crate::{
    __dep__, __dep__::StreamWriter,
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
            {__headers__}"
        ), content: None }
    }

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

impl<T: Serialize> Response<T> {
    fn to_string(self) -> String {
        match self {
            Self::Ok(res, _) => res,
            Self::Err(ErrResponse { status_and_headers, content:None }) => status_and_headers,
            Self::Err(ErrResponse { status_and_headers: mut res, content: Some((content_type, body)) }) => {
                res.push_str("\r\nContent-Type: ");
                res.push_str(content_type.as_str());
                res.push_str("\r\n\r\n");
                res.push_str(&body);
                res
            }
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
            {__headers__}\r\n\
            Content-Type: {__content_type__}\r\n\
            Content-Length: {__content_length__}\r\n\
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
            {__headers__}\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {__content_length__}\r\n\
            \r\n\
            {__body__}"
        ), PhantomData)
    }
}
impl Response<()> {
    #[inline(always)] pub(crate) fn ok_without_body(
        status: Status,
        headers: &ResponseHeaders,
    ) -> Self {
        let __status__ = status.as_str();
        let __headers__ = headers.to_string();

        Self::Ok(format!(
            "HTTP/1.1 {__status__}\r\n\
            {__headers__}"
        ), PhantomData)
    }
}
