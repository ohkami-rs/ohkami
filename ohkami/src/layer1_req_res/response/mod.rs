#[cfg(feature="nightly")]
use std::{
    ops::FromResidual,
    convert::Infallible
};

use std::{borrow::Cow};
use crate::{
    __rt__::AsyncWriter,
    layer0_lib::{Status, server_header},
};


/// # HTTP Response
/// 
/// Generated from `Context`. Handlers have to returns this.
/// 
/// ```
/// use ohkami::prelude::*;
/// 
/// async fn hello(c: Context) -> Response {
///     c
///         .OK()           // generate Response
///         .text("Hello!") // set content (text/plain)
/// }
/// ```
pub struct Response {
    pub status:         Status,
    pub(crate) headers: server_header::Headers,
    pub(crate) content: Option<Cow<'static, [u8]>>,
} const _: () = {
    #[cfg(feature="nightly")]
    impl FromResidual<Result<Infallible, Response>> for Response {
        fn from_residual(residual: Result<Infallible, Response>) -> Self {
            unsafe {residual.unwrap_err_unchecked()}
        }
    }
};

impl Response {
    pub(crate) fn into_bytes(self) -> Vec<u8> {
        let Self { status, headers, content } = self;

        let mut buf = Vec::from("HTTP/1.1 ");
        buf.extend_from_slice(status.as_bytes());
        buf.extend_from_slice(b"\r\n");
        headers.write_to(&mut buf);
        if let Some(body) = content {
            buf.extend_from_slice(&body);
        }
        
        buf
    }
}

impl Response {
    pub(crate) async fn send(self, stream: &mut (impl AsyncWriter + Unpin)) {
        if let Err(e) = stream.write_all(&self.into_bytes()).await {
            panic!("Failed to send response: {e}")
        }
    }
}

impl Response {
    pub fn drop_content(mut self) -> Self {
        self.content.take();
        self.headers.ContentType(None).ContentLength(None);
        self
    }

    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Self {
        let body = text.into();

        self.headers.ContentType("text/plain").ContentLength(body.len().to_string());
        self.content = Some(match body {
            Cow::Borrowed(s)   => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        });
        self
    }
    pub fn html(mut self, html: impl Into<Cow<'static, str>>) -> Self {
        let body = html.into();

        self.headers.ContentType("text/html").ContentLength(body.len().to_string());
        self.content = Some(match body {
            Cow::Borrowed(s)   => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        });
        self
    }
    pub fn json(mut self, json: impl serde::Serialize) -> Self {
        #[cold] fn __json_serialize_error_response(mut res: Response, err: serde_json::Error) -> Response {
            let body = err.to_string().into_bytes();
            res.headers.ContentType("text/plain").ContentLength(body.len().to_string());
            res.content = Some(Cow::Owned(body));
            res
        }

        match serde_json::to_string(&json) {
            Ok(json) => {let body = json.into_bytes();
                self.headers.ContentType("application/json").ContentLength(body.len().to_string());
                self.content = Some(Cow::Owned(body));
                self
            }
            Err(err) => __json_serialize_error_response(self, err)
        }
    }
    pub fn json_literal(mut self, json_literal: &'static str) -> Self {
        let body = json_literal.as_bytes();

        self.headers.ContentType("application/json").ContentLength(body.len().to_string());
        self.content = Some(Cow::Borrowed(body));
        self
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
                    .field("content", &*cow)
                    .finish(),
            }
        }
    }
};
