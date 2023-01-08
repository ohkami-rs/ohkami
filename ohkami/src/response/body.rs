use crate::{components::json::Json, prelude::Result};
use super::{message::Message, format::ResponseFormat};


/// Type of HTTP response body
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Body {
    application_json(String),
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
    /// Generate a `Body` that holds `application/json` response body.
    /// Types that implment `ToString` can be this' argument.
    pub fn json<Msg: Message>(text: Msg) -> Self {
        Self::application_json(text.as_message())
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
            Self::application_json(json) => json.len(),
            Self::text_plain(text) => text.len(),
            Self::text_html(html) => html.len(),
        }
    }
}

impl ResponseFormat for Body {
    fn response_format(&self) -> &str {
        match self {
            Self::application_json(json_str) => json_str.as_str(),
            Self::text_plain(text) => text.as_str(),
            Self::text_html(html) => html.as_str(),
        }
    }
}


pub trait IntoOK {fn into_ok(self) -> Result<Option<Body>>;}

impl IntoOK for String {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::text_plain(self)))
    }
}
impl IntoOK for Option<String> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(self.map(|string| Body::text_plain(string)))
    }
}
impl IntoOK for Result<String> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::text_plain(self?)))
    }
}

impl IntoOK for &str {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::text_plain(self.to_owned())))
    }
}
impl IntoOK for Option<&str> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(self.map(|string| Body::text_plain(string.to_owned())))
    }
}
impl IntoOK for Result<&str> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::text_plain(self?.to_owned())))
    }
}

impl<J: for <'j> Json<'j>> IntoOK for J {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::application_json(self.ser()?)))
    }
}
impl<J: for <'j> Json<'j>> IntoOK for Option<J> {
    fn into_ok(self) -> Result<Option<Body>> {
        match self {
            Some(json) => Ok(Some(Body::application_json(json.ser()?))),
            None => Ok(None),
        }
    }
}
impl<J: for <'j> Json<'j>> IntoOK for Result<J> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::application_json(self?.ser()?)))
    }
}

impl IntoOK for Body {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(self))
    }
}
impl IntoOK for Option<Body> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(self)
    }
}
impl IntoOK for Result<Body> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(self?))
    }
}


pub trait IntoCreated {fn into_created(self) -> Result<Body>;}

impl<J: for <'j> Json<'j>> IntoCreated for J {
    fn into_created(self) -> Result<Body> {
        Ok(Body::application_json(self.ser()?))
    }
}
impl<J: for <'j> Json<'j>> IntoCreated for Result<J> {
    fn into_created(self) -> Result<Body> {
        Ok(Body::application_json(self?.ser()?))
    }
}

impl IntoCreated for Body {
    fn into_created(self) -> Result<Body> {
        Ok(self)
    }
}
impl IntoCreated for Result<Body> {
    fn into_created(self) -> Result<Body> {
        self
    }
}
