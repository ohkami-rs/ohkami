#![allow(non_snake_case)]

use std::borrow::Cow;
use crate::{Request, Response, layer0_lib::{server_header, Status}};


pub trait Responder {
    fn respond_to(self, req: &Request) -> Response;
}


impl Responder for Response {
    fn respond_to(self, _: &Request) -> Response {
        self
    }
}
impl<T:Responder, E:Responder> Responder for Result<T, E> {
    fn respond_to(self, req: &Request) -> Response {
        match self {
            Ok(ok) => ok.respond_to(req),
            Err(e) => e.respond_to(req),
        }
    }
}


pub struct JSON<T: serde::Serialize> {
    status: Status,
    body:   T,
}
impl<T: serde::Serialize> Responder for JSON<T> {
    fn respond_to(self, _: &Request) -> Response {
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
impl Responder for Text {
    fn respond_to(self, _: &Request) -> Response {
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
impl Responder for HTML {
    fn respond_to(self, _: &Request) -> Response {
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
impl Responder for Redirect {
    fn respond_to(self, _: &Request) -> Response {
        self.into()
    }
}


pub struct Empty {
    status: Status
}
macro_rules! generate_empty_response {
    ($( $status:ident, )*) => {
        impl Empty {$(
            pub fn $status() -> Self {
                Self {
                    status: Status::$status,
                }
            }
        )*}
    };
} generate_empty_response! {
    OK,
    Created,
    NoContent,

    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,

    InternalServerError,
    NotImplemented,
}
impl Into<Response> for Empty {
    fn into(self) -> Response {
        Response::NoContent()
    }
}
impl Responder for Empty {
    fn respond_to(self, _: &Request) -> Response {
        self.into()
    }
}
