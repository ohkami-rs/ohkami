use std::borrow::Cow;

use crate::{components::json::{JSON, JsonResponse, JsonResponseLabel}, prelude::Result};
use super::{message::Message, format::ResponseFormat};


/// Type of HTTP response body
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Body {
    application_json(Cow<'static, str>),
    text_plain(Cow<'static, str>),
    text_html(Cow<'static, str>),
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
            Self::application_json(json_str) => match json_str {
                Cow::Borrowed(str) => str,
                Cow::Owned(string) => &string,
            },
            Self::text_plain(text) => match text {
                Cow::Borrowed(str) => str,
                Cow::Owned(string) => &string,
            },
            Self::text_html(html) => match html {
                Cow::Borrowed(str) => str,
                Cow::Owned(string) => &string,
            },
        }
    }
}




pub trait IntoOK<OkParam> {fn into_ok(self) -> Result<Option<Body>>;}

impl IntoOK<String> for String {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::text_plain(Cow::Owned(self))))
    }
}
impl IntoOK<Option<String>> for Option<String> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(self.map(|string| Body::text_plain(Cow::Owned(string))))
    }
}
impl IntoOK<Result<String>> for Result<String> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::text_plain(Cow::Owned(self?))))
    }
}


impl IntoOK<&String> for &String {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::text_plain(Cow::Owned(self.to_owned()))))
    }
}
impl IntoOK<&'static str> for &'static str {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::text_plain(Cow::Borrowed(self))))
    }
}
impl IntoOK<Option<&String>> for Option<&String> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(self.map(|string| Body::text_plain(Cow::Owned(string.to_owned()))))
    }
}
impl IntoOK<Option<&'static str>> for Option<&'static str> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(self.map(|string| Body::text_plain(Cow::Borrowed(string))))
    }
}
impl IntoOK<Result<&String>> for Result<&String> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::text_plain(Cow::Owned(self?.to_owned()))))
    }
}
impl IntoOK<Result<&'static str>> for Result<&'static str> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::text_plain(Cow::Borrowed(self?))))
    }
}

impl<L: JsonResponseLabel, J: JsonResponse<L>> IntoOK<L> for J {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::application_json(Cow::Owned(self.ser()?))))
    }
}
impl<L: JsonResponseLabel, J: JsonResponse<L>> IntoOK<Option<L>> for Option<J> {
    fn into_ok(self) -> Result<Option<Body>> {
        match self {
            Some(json) => Ok(Some(Body::application_json(Cow::Owned(json.ser()?)))),
            None => Ok(None),
        }
    }
}
impl<L: JsonResponseLabel, J: JsonResponse<L>> IntoOK<Result<L>> for Result<J> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(Body::application_json(Cow::Owned(self?.ser()?))))
    }
}

impl IntoOK<Body> for Body {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(self))
    }
}
impl IntoOK<Body> for Option<Body> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(self)
    }
}
impl IntoOK<Body> for Result<Body> {
    fn into_ok(self) -> Result<Option<Body>> {
        Ok(Some(self?))
    }
}




pub trait IntoCreated {fn into_created(self) -> Result<Body>;}

impl<J: for <'j> JSON<'j>> IntoCreated for J {
    fn into_created(self) -> Result<Body> {
        Ok(Body::application_json(Cow::Owned(self.ser()?)))
    }
}
impl<J: for <'j> JSON<'j>> IntoCreated for Result<J> {
    fn into_created(self) -> Result<Body> {
        Ok(Body::application_json(Cow::Owned(self?.ser()?)))
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
