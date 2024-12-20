use crate::{FromRequest, Request, Response};
use super::bound::Incoming;

#[cfg(all(debug_assertions, feature="openapi"))]
use crate::openapi;


pub use ohkami_lib::serde_multipart::File;

pub struct Multipart<T>(pub T);

impl<'req, T: Incoming<'req>> FromRequest<'req> for Multipart<T> {
    type Error = Response;

    #[inline]
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if req.headers.ContentType()? != "multipart/form-data" {
            return None
        }
        ohkami_lib::serde_multipart::from_bytes(req.payload()?)
            .map_err(super::reject)
            .map(Self).into()
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_input() -> Option<openapi::Input> {
        Some(openapi::Input::Body(openapi::RequestBody::of(
            "multipart/form-data",
            T::schema()
        )))
    }
}
