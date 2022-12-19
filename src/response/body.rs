use crate::components::json::JSON;
use super::{format::ResponseFormat, message::Message};


/// Type of HTTP response body
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Body {
    application_json(JSON),
    text_plain(String),
    text_html(String),
} impl Body {
    /// Generate a `Body` that holds `text/plain` response body.
    /// Types that implment `ToString` can be this' argument.
    pub fn text<Msg: Message>(text: Msg) -> Self {
        Self::text_plain(text.as_message())
    }
    /// Generate a `Body` that holds `text/html` response body.
    /// Types that implment `ToString` can be this' argument.
    pub fn html<Msg: Message>(html: Msg) -> Self {
        Self::text_html(html.as_message())
    }

    pub(crate) fn content_type(&self) -> &'static str {
        match self {
            Self::application_json(_) => "application/json",
            Self::text_plain(_) => "text/plain",
            Self::text_html(_) => "text/html",
        }
    }
    pub(crate) fn content_length(&self) -> usize {
        match self {
            Self::application_json(json) => json.content_length(),
            Self::text_plain(text) => text.len(),
            Self::text_html(html) => html.len(),
        }
    }
}

impl Into<Body> for JSON {
    fn into(self) -> Body {
        Body::application_json(self)
    }
}
impl Into<Body> for String {
    fn into(self) -> Body {
        Body::text_plain(self)
    }
}
impl Into<Body> for &str {
    fn into(self) -> Body {
        Body::text_plain(self.to_owned())
    }
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


pub trait ResponseBody {fn as_body(self) -> Option<Body>;}
impl ResponseBody for Body {fn as_body(self) -> Option<Body> {Some(self)}}
impl ResponseBody for Option<Body> {fn as_body(self) -> Option<Body> {self}}