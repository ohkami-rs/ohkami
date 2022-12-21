use async_std::{net::TcpStream, io::WriteExt};
use crate::{
    components::{
        status::Status,
        json::JSON, headers::Header, time::now_fmt,
    },
    result::Result,
};

pub(crate) mod body;
use body::Body;

pub(crate) mod format;
use format::ResponseFormat;

mod message;
use message::Message;

use self::{body::ResponseBody, message::ErrorMessage};


/// Type of HTTP response
#[derive(Debug, PartialEq)]
pub struct Response {
    additional_headers: String,
    status: Status,
    body:   Option<Body>,
} impl Response {
    /// Add error context message to an existing `Response` in `Err`.
    /// ```no_run
    /// let requested_user = ctx.body::<User>()
    ///     ._else(|err| err.error_context("can't deserialize user"))?;
    /// ```
    pub fn error_context<Msg: Message>(mut self, msg: Msg) -> Self {
        use Status::*;
        match self.status {
            OK | Created => unreachable!(),
            _ => match self.body {
                Some(Body::application_json(_)) => unreachable!(),
                Some(Body::text_plain(ref mut t)) => {
                    *t = format!("{}: ", msg.as_message()) + t;
                    self
                },
                Some(Body::text_html(ref mut t)) => {
                    *t = format!("{}: ", msg.as_message()) + t;
                    self
                },
                None => {
                    self.body = Some(Body::text_plain(msg.as_message()));
                    self
                },
            }
        }
    }

    pub(crate) async fn write_to_stream(self, stream: &mut TcpStream) -> async_std::io::Result<usize> {
        stream.write(
            match self.body {
                Some(body) => format!(
"HTTP/1.1 {}
Connection: Keep-Alive
Content-Type: {}; charset=utf-8
Content-Length: {}
Server: ohkami
Date: {}
Keep-Alive: timeout=5
{}
{}",
                    self.status.response_format(),
                    body.content_type(),
                    body.content_length(),
                    now_fmt(),
                    self.additional_headers,
                    body.response_format(),
                ),
                None => format!(
"HTTP/1.1 {}
Connection: Keep-Alive
Content-Length: 0
Server: ohkami
Date: {}
Keep-Alive: timeout=5
{}
",
                    self.status.response_format(),
                    now_fmt(),
                    self.additional_headers,
                ),
            }
        .as_bytes()).await
    }
    pub(crate) fn add_header(&mut self, key: Header, value: &String) {
        self.additional_headers += key.response_format();
        self.additional_headers += value;
        self.additional_headers += "\n";
    }

    /// Generate `Result<Response>` value that represents a HTTP response of `200 OK`. `JSON`, `String`, `&str`, or `Option` of them can be argument of this.\
    /// You can directly return `Response::OK(/* something */)` from a handler because this is already wrapped in `Result::Ok`.
    #[allow(non_snake_case)]
    pub fn OK<B: ResponseBody>(body: B) -> Result<Self> {
        Ok(Self {
            additional_headers: String::new(),
            status:             Status::OK,
            body:               body.as_body(),
        })
    }
    /// Generate `Result<Response>` value that represents a HTTP response of `201 Created`.
    /// You can directly return `Response::Created(/* something */)` from a handler because this is already wrapped in `Result::Ok`.
    #[allow(non_snake_case)]
    pub fn Created(body: JSON) -> Result<Self> {
        Ok(Self {
            additional_headers: String::new(),
            status:             Status::Created,
            body:               Some(Body::application_json(body)),
        })
    }

    /// Generate `Response` value that represents a HTTP response of `404 Not Found`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn NotFound<Msg: ErrorMessage>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status:             Status::NotFound,
            body:               msg.as_message(),
        }
    }
    /// Generate `Response` value that represents a HTTP response of `400 Bad Request`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn BadRequest<Msg: ErrorMessage>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status:             Status::BadRequest,
            body:               msg.as_message(),
        }
    }
    /// Generate `Response` value that represents a HTTP response of `500 Internal Server Error`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn InternalServerError<Msg: ErrorMessage>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status:             Status::InternalServerError,
            body:               msg.as_message(),
        }
    }
    /// Generate `Response` value that represents a HTTP response of `501 Not Implemented`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn NotImplemented<Msg: ErrorMessage>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status:             Status::NotImplemented,
            body:               msg.as_message(),
        }
    }
    /// Generate `Response` value that represents a HTTP response of `403 Forbidden`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn Forbidden<Msg: ErrorMessage>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status:             Status::Forbidden,
            body:               msg.as_message(),
        }
    }
    /// Generate `Response` value that represents a HTTP response of `401 Unauthorized`.
    /// `String`, `&str` or `Option<String>` can be argument of this.
    #[allow(non_snake_case)]
    pub fn Unauthorized<Msg: ErrorMessage>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status:             Status::Unauthorized,
            body:               msg.as_message(),
        }
    }
}
