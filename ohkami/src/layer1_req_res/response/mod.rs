mod headers;
pub use headers::{Headers as ResponseHeaders};
#[cfg(feature="testing")]
pub(crate) use headers::{Header as ResponseHeader};

mod into_response;
pub use into_response::IntoResponse;

#[cfg(feature="nightly")]
use std::{
    ops::FromResidual,
    convert::Infallible
};

use std::{
    borrow::Cow,
};
use crate::{
    __rt__::AsyncWriter,
    layer0_lib::Status,
};


pub struct Response {
    pub status:         Status,
    pub headers:        ResponseHeaders,
    pub(crate) content: Option<Cow<'static, [u8]>>,
} const _: () = {
    #[cfg(feature="nightly")]
    impl FromResidual<Result<Infallible, Response>> for Response {
        fn from_residual(residual: Result<Infallible, Response>) -> Self {
            unsafe {residual.unwrap_err_unchecked()}
        }
    }
};

macro_rules! new_response {
    ($( $status:ident, )*) => {
        #[allow(non_snake_case)]
        impl Response {$(
            pub fn $status() -> Self {
                Self {
                    status:  Status::$status,
                    headers: ResponseHeaders::new(),
                    content: None,
                }
            }
        )*}
    };
} new_response! {
    SwitchingProtocols,

    OK,
    Created,
    NoContent,

    MovedPermanently,
    Found,

    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    UnprocessableEntity,

    InternalServerError,
    NotImplemented,
}

impl Response {
    #[inline] pub(crate) fn into_bytes(self) -> Vec<u8> {
        let Self { status, headers, content, .. } = self;

        let mut buf = Vec::from("HTTP/1.1 ");
        buf.extend_from_slice(status.as_bytes());
        buf.extend_from_slice(b"\r\n");
        headers.write_to(&mut buf);
        if let Some(content) = content {
            buf.extend_from_slice(&content);
        }
        
        buf
    }
}

impl Response {
    #[inline] pub(crate) async fn send(self, stream: &mut (impl AsyncWriter + Unpin)) {
        if let Err(e) = stream.write_all(&self.into_bytes()).await {
            panic!("Failed to send response: {e}")
        }
    }
}

impl Response {
    pub fn drop_content(mut self) -> Self {
        self.content = None;
        self.headers.set()
            .ContentType(None)
            .ContentLength(None);
        self
    }

    pub fn text<Text: Into<Cow<'static, str>>>(mut self, text: Text) -> Response {
        let body = text.into();

        self.headers.set()
            .ContentType("text/plain; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(match body {
            Cow::Borrowed(s)   => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        });

        Response {
            status:       self.status,
            headers:      self.headers,
            content:      self.content,
        }
    }
    pub fn html<HTML: Into<Cow<'static, str>>>(mut self, html: HTML) -> Response {
        let body = html.into();

        self.headers.set()
            .ContentType("text/html; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(match body {
            Cow::Borrowed(s)   => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        });

        Response {
            status:       self.status,
            headers:      self.headers,
            content:      self.content,
        }
    }
    pub fn json<JSON: serde::Serialize>(mut self, json: JSON) -> Response {
        let body = ::serde_json::to_vec(&json).unwrap();

        self.headers.set()
            .ContentType("application/json; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(Cow::Owned(body));

        Response {
            status:       self.status,
            headers:      self.headers,
            content:      self.content,
        }
    }
    pub fn json_literal<JSONString: Into<Cow<'static, str>>>(mut self, json_literal: JSONString) -> Response {
        let body = match json_literal.into() {
            Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        };

        self.headers.set()
            .ContentType("application/json; charset=UTF-8")
            .ContentLength(body.len().to_string());
        self.content = Some(body);

        Response {
            status:       self.status,
            headers:      self.headers,
            content:      self.content,
        }
    }
}

const _: () = {
    impl std::fmt::Debug for Response {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.content {
                None => f.debug_struct("Response")
                    .field("status",  &self.status)
                    .field("headers", &self.headers)
                    .finish(),
                Some(cow) => f.debug_struct("Response")
                    .field("status",  &self.status)
                    .field("headers", &self.headers)
                    .field("content", &cow.escape_ascii())
                    .finish(),
            }
        }
    }
};
