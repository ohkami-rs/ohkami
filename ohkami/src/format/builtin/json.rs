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
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
            .and_then(super::super::validated)
            .map(Self).into()
    }
}

impl<S: Serialize> IntoResponse for JSON<S> {
    #[inline(always)]
    fn into_response(self) -> Response {
        match super::super::validated(self.0) {
            Ok(v)  => Response::OK().with_json(v),
            Err(e) => e,
        }
    }
}
