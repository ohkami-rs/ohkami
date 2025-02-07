use crate::{FromBody, IntoBody};
use std::borrow::Cow;

#[cfg(feature="openapi")]
use crate::openapi;


pub struct Text<T>(pub T);

impl<'req, T: From<&'req str>> FromBody<'req> for Text<T> {
    const MIME_TYPE: &'static str = "text/plain";

    fn from_body(body: &'req [u8]) -> Result<Self, impl std::fmt::Display> {
        std::str::from_utf8(body).map(|s| Text(s.into()))
    }

    #[cfg(feature="openapi")]
    fn openapi_requestbody() -> impl Into<openapi::schema::SchemaRef> {
        openapi::string()
    }
}

impl<T: Into<Cow<'static, str>>> IntoBody for Text<T> {
    const CONTENT_TYPE: &'static str = "text/plain; charset=UTF-8";

    fn into_body(self) -> Result<Cow<'static, [u8]>, impl std::fmt::Display> {
        Result::<_, std::convert::Infallible>::Ok(match self.0.into() {
            Cow::Owned(s) => Cow::Owned(s.into_bytes()),
            Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
        })
    }

    #[cfg(feature="openapi")]
    fn openapi_responsebody() -> impl Into<openapi::schema::SchemaRef> {
        openapi::string()
    }
}
