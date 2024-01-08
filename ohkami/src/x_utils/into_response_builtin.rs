#![allow(non_snake_case)]
use crate::{Response, IntoResponse, http::Status, layer0_lib::server_header};
use std::borrow::Cow;


pub struct JSON<T: serde::Serialize> {
    status: Status,
    body:   T,
}
impl<'req, T: serde::Serialize> IntoResponse for JSON<T> {
    fn into_response(self) -> Response {
        self.into()
    }
}
impl<T: serde::Serialize> Into<Response> for JSON<T> {
    fn into(self) -> Response {
        let body = serde_json::to_vec(&self.body).unwrap();

        let mut headers = server_header::Headers::new();
        headers.set()
            .ContentType("application/json; charset=UTF-8")
            .ContentLength(body.len().to_string());

        Response {
            headers,
            status:  self.status,
            content: Some(Cow::Owned(body)),
        }
    }
}
macro_rules! generate_json_response {
    ($( $status:ident, )*) => {
        impl<T: serde::Serialize> JSON<T> {$(
            pub fn $status(body: T) -> Self {
                Self {
                    status: Status::$status,
                    body
                }
            }
        )*}
    };
} generate_json_response! {
    OK,
    Created,

    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,

    InternalServerError,
}


pub struct Text {
    status:  Status,
    content: Cow<'static, str>,
}
macro_rules! generate_text_response {
    ($( $status:ident, )*) => {
        impl Text {$(
            pub fn $status(text: impl Into<Cow<'static, str>>) -> Self {
                Self {
                    status:  Status::$status,
                    content: text.into(),
                }
            }
        )*}
    };
} generate_text_response! {
    OK,
    Created,

    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,

    InternalServerError,
}
impl Into<Response> for Text {
    fn into(self) -> Response {
        let content = match self.content {
            Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes())
        };

        let mut headers = server_header::Headers::new();
        headers.set()
            .ContentType("text/plain; charset=UTF-8")
            .ContentLength(content.len().to_string());

        Response {
            headers,
            status:  self.status,
            content: Some(content),
        }
    }
}
impl<'req> IntoResponse for Text {
    fn into_response(self) -> Response {
        self.into()
    }
}


pub struct HTML {
    status:  Status,
    content: Cow<'static, str>,
}
macro_rules! generate_text_response {
    ($( $status:ident, )*) => {
        impl HTML {$(
            pub fn $status(html: impl Into<Cow<'static, str>>) -> Self {
                Self {
                    status:  Status::$status,
                    content: html.into(),
                }
            }
        )*}
    };
} generate_text_response! {
    OK,
    Created,

    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,

    InternalServerError,
}
impl Into<Response> for HTML {
    fn into(self) -> Response {
        let content = match self.content {
            Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes())
        };

        let mut headers = server_header::Headers::new();
        headers.set()
            .ContentType("text/html; charset=UTF-8")
            .ContentLength(content.len().to_string());

        Response {
            headers,
            status:  self.status,
            content: Some(content),
        }
    }
}
impl<'req> IntoResponse for HTML {
    fn into_response(self) -> Response {
        self.into()
    }
}


pub struct Redirect {
    location: Cow<'static, str>,
}
impl Redirect {
    pub fn to(location: impl Into<Cow<'static, str>>) -> Self {
        Self { location: location.into() }
    }
}
impl Into<Response> for Redirect {
    fn into(self) -> Response {
        let mut headers = server_header::Headers::new();
        headers.set().Location(self.location);
        Response {
            status:  Status::Found,
            content: None,
            headers,
        }
    }
}
impl<'req> IntoResponse for Redirect {
    fn into_response(self) -> Response {
        self.into()
    }
}
