use crate::FromBody;
use super::bound::{self, Incoming};
use ohkami_lib::serde_multipart;

#[cfg(feature="openapi")]
use crate::openapi;


pub use ohkami_lib::serde_multipart::File;

pub struct Multipart<T: bound::Schema>(pub T);

impl<'req, T: Incoming<'req>> FromBody<'req> for Multipart<T> {
    const MIME_TYPE: &'static str = "multipart/form-data";

    fn from_body(body: &'req [u8]) -> Result<Self, impl std::fmt::Display> {
        serde_multipart::from_bytes(body).map(Multipart)
    }

    #[cfg(feature="openapi")]
    fn openapi_requestbody() -> impl Into<openapi::schema::SchemaRef> {
        T::schema()
    }
}
