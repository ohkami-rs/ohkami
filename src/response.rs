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
    enum Body {
        json(JSON),
        text(String),
    } impl Body {
        fn content_length(&self) -> usize {
            match self {
                Self::json(json) => json.content_length(),
                Self::text(text) => text.len(),
            }
        }
    }

pub(crate) trait ResponseFormat {
    fn response_format(&self) -> &str;
}
impl ResponseFormat for Body {
    fn response_format(&self) -> &str {
        match self {
            Self::json(json) => json.response_format(),
            Self::text(text) => text.as_str(),
        }
    }
}

impl Response {
    pub fn error_context<Msg: ToString>(mut self, msg: Msg) -> Self {
        use Status::*;
        match self.status {
            OK | Created => unreachable!(),
            _ => match self.body {
                Body::json(_) => unreachable!(),
                Body::text(ref mut t) => {
                    *t += &format!("{}: ", msg.to_string());
                    self
                }
            }
        }
    }

    pub(crate) async fn write_to_stream(self, stream: &mut TcpStream) -> async_std::io::Result<usize> {
        stream.write(format!(
"HTTP/1.1 {}
Connection: Keep-Alive
Content-Type: {}; charset=utf-8
Content-Length: {}
Date: {}
Keep-Alive: timeout=5
{}
{}",
            self.status.response_format(),
            self.status.content_type(),
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
    pub(crate) fn SetUpError(messages: &Vec<String>) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::SetUpError,
            body:   Body::text(messages.iter().fold(
                String::new(), |a, b| a + b + "\n"
            ))
        }
    }

    #[allow(non_snake_case)]
    pub fn OK(body: JSON) -> Result<Self> {
        Ok(Self {
            additional_headers: String::new(),
            status: Status::OK,
            body:   Body::json(body),
        })
    }
    #[allow(non_snake_case)]
    pub fn Created(body: JSON) -> Result<Self> {
        Ok(Self {
            additional_headers: String::new(),
            status: Status::Created,
            body:   Body::json(body),
        })
    }


    #[allow(non_snake_case)]
    pub fn NotFound<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::NotFound,
            body:   Body::text(msg.to_string()),
        }
    }
    #[allow(non_snake_case)]
    pub fn BadRequest<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::BadRequest,
            body:   Body::text(msg.to_string())
        }
    }
    #[allow(non_snake_case)]
    pub fn InternalServerError<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::InternalServerError,
            body:   Body::text(msg.to_string()),
        }
    }
    #[allow(non_snake_case)]
    pub fn NotImplemented<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::NotImplemented,
            body:   Body::text(msg.to_string()),
        }
    }
    #[allow(non_snake_case)]
    pub fn Forbidden<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::Forbidden,
            body:   Body::text(msg.to_string()),
        }
    }
    #[allow(non_snake_case)]
    pub fn Unauthorized<Msg: ToString>(msg: Msg) -> Self {
        Self {
            additional_headers: String::new(),
            status: Status::Unauthorized,
            body:   Body::text(msg.to_string()),
        }
    }
}
