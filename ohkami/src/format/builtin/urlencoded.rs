use crate::{FromBody, IntoBody};
use super::bound::{self, Incoming, Outgoing};
use ohkami_lib::serde_urlencoded;
use std::borrow::Cow;

#[cfg(feature="openapi")]
use crate::openapi;


pub struct URLEncoded<T: bound::Schema>(pub T);

impl<'req, T: Incoming<'req>> FromBody<'req> for URLEncoded<T> {
    const MIME_TYPE: &'static str = "application/x-www-form-urlencoded";

    fn from_body(body: &'req [u8]) -> Result<Self, impl std::fmt::Display> {
        serde_urlencoded::from_bytes(body).map(URLEncoded)
    }

    #[cfg(feature="openapi")]
    fn openapi_requestbody() -> impl Into<openapi::schema::SchemaRef> {
        T::schema()
    }
}

impl<T: Outgoing> IntoBody for URLEncoded<T> {
    const CONTENT_TYPE: &'static str = "application/x-www-form-urlencoded";

    fn into_body(self) -> Result<Cow<'static, [u8]>, impl std::fmt::Display> {
        serde_urlencoded::to_string(&self.0).map(|s| Cow::Owned(s.into_bytes()))
    }

    #[cfg(feature="openapi")]
    fn openapi_responsebody() -> impl Into<openapi::schema::SchemaRef> {
        T::schema()
    }
}
