#![allow(non_snake_case)]

use crate::{Response, Status};
use crate::typed::{ResponseBody, body_type};
use crate::response::ResponseHeaders;
use crate::serde::Serialize;
use std::borrow::Cow;


/// Clone on write `text/plain` response
pub fn Text(text: impl Into<Cow<'static, str>>) -> Text {
    Text {
        content: text.into()
    }
}
pub struct Text {
    content: Cow<'static, str>,
}
impl Serialize for Text {
    #[inline] fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.content.serialize(serializer)    
    }
}
impl ResponseBody for Text {
    type Type = body_type::Text;
    #[inline] fn into_response_with(self, status: Status) -> Response {
        let content = match self.content {
            Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes())
        };

        let mut headers = ResponseHeaders::new();
        headers.set()
            .ContentType("text/plain; charset=UTF-8")
            .ContentLength(content.len().to_string());

        Response {
            status,
            headers,
            content: Some(content),
        }
    }
}

/// Clone on write `text/html` response
pub fn HTML(text: impl Into<Cow<'static, str>>) -> HTML {
    HTML {
        content: text.into()
    }
}
pub struct HTML {
    content: Cow<'static, str>,
}
impl Serialize for HTML {
    #[inline] fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.content.serialize(serializer)    
    }
}
impl ResponseBody for HTML {
    type Type = body_type::HTML;
    #[inline] fn into_response_with(self, status: Status) -> Response {
        let content = match self.content {
            Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes())
        };

        let mut headers = ResponseHeaders::new();
        headers.set()
            .ContentType("text/html; charset=UTF-8")
            .ContentLength(content.len().to_string());

        Response {
            status,
            headers,
            content: Some(content),
        }
    }
}
