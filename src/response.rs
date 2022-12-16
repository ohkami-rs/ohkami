use async_std::{net::TcpStream, io::WriteExt};
use chrono::Utc;
use crate::{
    components::{
        status::Status,
        json::JSON
    },
    result::Result,
};


#[derive(Debug)]
pub struct Response {
    additional_headers: String,
    status: Status,
    body:   Body,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Body {
    application_json(JSON),
    text_plain(String),
    text_html(String),
} impl Body {
    pub fn text<Str: ToString>(text: Str) -> Self {
        Self::text_plain(text.to_string())
    }
    pub fn html<Str: ToString>(html: Str) -> Self {
        Self::text_html(html.to_string())
    }

    fn content_type(&self) -> &'static str {
        match self {
            Self::application_json(_) => "application/json",
            Self::text_plain(_) => "text/plain",
            Self::text_html(_) => "text/html",
        }
    }
    fn content_length(&self) -> usize {
        match self {
            Self::application_json(json) => json.content_length(),
            Self::text_plain(text) => text.len(),
            Self::text_html(html) => html.len(),
        }
    }
} impl Into<Body> for JSON {
    fn into(self) -> Body {
        Body::application_json(self)
    }
}

pub(crate) trait ResponseFormat {
    fn response_format(&self) -> &str;
}
impl ResponseFormat for Body {
    fn response_format(&self) -> &str {
        match self {
            Self::application_json(json) => json.response_format(),
            Self::text_plain(text) => text.as_str(),
            Self::text_html(html) => html.as_str(),
        }
    }
}

impl Response {
    pub fn error_context<Msg: ToString>(mut self, msg: Msg) -> Self {
        use Status::*;
        match self.status {
            OK | Created => unreachable!(),
            _ => match self.body {
                Body::application_json(_) => unreachable!(),
                Body::text_plain(ref mut t) => {
                    *t = format!("{}: ", msg.to_string()) + t;
                    self
                },
                Body::text_html(ref mut t) => {
                    *t = format!("{}: ", msg.to_string()) + t;
                    self
                },
            }
        }
    }

    pub(crate) async fn write_to_stream(self, stream: &mut TcpStream) -> async_std::io::Result<usize> {
        stream.write(format!(
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
            self.body.content_type(),
            self.body.content_length(),
            Utc::now().to_rfc2822(),
            self.additional_headers,
            self.body.response_format(),
        ).as_bytes()).await
    }
    pub(crate) fn add_header(&mut self, header_string: &String) {
        self.additional_headers += header_string;
        self.additional_headers += "\n";
    }

    #[allow(non_snake_case)]
    pub fn OK<B: Into<Body>>(body: B) -> Result<Self> {
        Ok(Self {
            additional_headers: String::new(),
            status: Status::OK,
            body:   body.into(),
        })
    }
    #[allow(non_snake_case)]
    pub fn Created(body: JSON) -> Result<Self> {
        Ok(Self {
            additional_headers: String::new(),
            status: Status::Created,
            body:   Body::application_json(body),
        })
    }

    #[allow(non_snake_case)]
    pub fn NotFound<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::NotFound,
            body:   Body::text_plain(msg.to_string()),
        }
    }
    #[allow(non_snake_case)]
    pub fn BadRequest<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::BadRequest,
            body:   Body::text_plain(msg.to_string())
        }
    }
    #[allow(non_snake_case)]
    pub fn InternalServerError<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::InternalServerError,
            body:   Body::text_plain(msg.to_string()),
        }
    }
    #[allow(non_snake_case)]
    pub fn NotImplemented<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::NotImplemented,
            body:   Body::text_plain(msg.to_string()),
        }
    }
    #[allow(non_snake_case)]
    pub fn Forbidden<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::Forbidden,
            body:   Body::text_plain(msg.to_string()),
        }
    }
    #[allow(non_snake_case)]
    pub fn Unauthorized<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::Unauthorized,
            body:   Body::text_plain(msg.to_string()),
        }
    }
}
