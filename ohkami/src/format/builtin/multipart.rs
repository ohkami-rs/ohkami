use crate::{FromRequest, Request, Response};
use serde::Deserialize;


pub use ohkami_lib::serde_multipart::File;

pub struct Multipart<Schema>(pub Schema);

impl<'req, S: Deserialize<'req>> FromRequest<'req> for Multipart<S> {
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
}
