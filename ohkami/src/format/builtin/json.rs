use crate::{FromRequest, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};


pub struct JSON<Schema>(pub Schema);

impl<'req, S: Deserialize<'req>> FromRequest<'req> for JSON<S> {
    type Error = Response;

    #[inline(always)]
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if req.headers.ContentType()? != "application/json" {
            return None
        }
        serde_json::from_slice(req.payload()?)
            .map_err(super::reject)
            .map(Self).into()
    }
}

impl<S: Serialize> IntoResponse for JSON<S> {
    #[inline(always)]
    fn into_response(self) -> Response {
        Response::OK().with_json(self.0)
    }
}
