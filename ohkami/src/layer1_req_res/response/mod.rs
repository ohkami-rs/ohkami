mod headers; pub(crate) use headers::ResponseHeaders;

use std::{
    borrow::Cow,
    ops::FromResidual,
    convert::Infallible,
};
use crate::{
    __dep__, __dep__::AsyncWriter,
    layer0_lib::{Status, ContentType, AsStr, IntoCow},
};


pub struct Response {
    pub(crate) status:  Status,
    pub(crate) headers: String,
    pub(crate) content: Option<(ContentType, Cow<'static, str>)>,
}

impl FromResidual<Result<Infallible, Response>> for Response {
    fn from_residual(residual: Result<Infallible, Response>) -> Self {
        unsafe { residual.unwrap_err_unchecked() }
    }
}

impl Response {
    fn into_bytes(self) -> Vec<u8> {
        let Self { status, headers, content } = self;
        let (status, headers) = (status.as_bytes(), headers.as_bytes());

        match content {
            None => [
                b"HTTP/1.1 ",status,b"\r\n",
                headers,
                b"\r\n"
            ].concat(),

            Some((content_type, body)) => [   
                b"HTTP/1.1 ",status,b"\r\n",
                b"Content-Type: "  ,content_type.as_bytes(),          b"\r\n",
                b"Content-Length: ",body.len().to_string().as_bytes(),b"\r\n",
                headers,
                b"\r\n",
                body.as_bytes()
            ].concat(),
        }
    }
}

impl Response {
    pub(crate) async fn send(self, stream: &mut __dep__::TcpStream) {
        if let Err(e) = stream.write_all(&self.into_bytes()).await {
            panic!("Failed to send response: {e}")
        }
    }
}

impl Response {
    pub fn text(mut self, text: impl IntoCow<'static>) -> Self {
        self.content.replace((
            ContentType::Text,
            text.into_cow()
        ));
        self
    }
    pub fn html(mut self, html: impl IntoCow<'static>) -> Self {
        self.content.replace((
            ContentType::HTML,
            html.into_cow()
        ));
        self
    }
    pub fn json(mut self, json: impl serde::Serialize) -> Self {
        self.content.replace((
            ContentType::JSON,
            Cow::Owned(serde_json::to_string(&json).expect("Failed to serialize json"))
        ));
        self
    }
}
