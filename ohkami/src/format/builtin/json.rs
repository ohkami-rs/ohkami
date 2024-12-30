use crate::{FromBody, IntoBody};
use super::bound::{self, Incoming, Outgoing};

#[cfg(feature="openapi")]
use crate::openapi;


pub struct JSON<T: bound::Schema>(pub T);

impl<'req, T: Incoming<'req>> FromBody<'req> for JSON<T> {
    const MIME_TYPE: &'static str = "application/json";

    #[inline]
    fn from_body(body: &'req [u8]) -> Result<Self, impl std::fmt::Display> {
        serde_json::from_slice(body).map(JSON)
    }

    #[cfg(feature="openapi")]
    fn openapi_requestbody() -> impl Into<openapi::schema::SchemaRef> {
        T::schema()
    }
}

impl<T: Outgoing> IntoBody for JSON<T> {
    const CONTENT_TYPE: &'static str = "application/json";

    #[inline]
    fn into_body(self) -> Result<Vec<u8>, impl std::fmt::Display> {
        serde_json::to_vec(&self.0)
    }

    #[cfg(feature="openapi")]
    fn openapi_responsebody() -> impl Into<openapi::schema::SchemaRef> {
        T::schema()
    }
}
