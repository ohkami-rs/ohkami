use crate::{FromRequest, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};


pub struct URLEncoded<Schema>(pub Schema);

impl<'req, S: Deserialize<'req>> FromRequest<'req> for URLEncoded<S> {
    type Error = Response;

    #[inline]
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if req.headers.ContentType()? != "application/x-www-form-urlencoded" {
            return None
        }
        ohkami_lib::serde_urlencoded::from_bytes(req.payload()?)
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
            .and_then(super::super::validated)
            .map(Self).into()
    }
}

impl<S: Serialize> IntoResponse for URLEncoded<S> {
    fn into_response(self) -> Response {
        match super::super::validated(self.0) {
            Ok(v) => Response::OK().with_payload("application/x-www-form-urlencoded",
                ohkami_lib::serde_urlencoded::to_string(&v).unwrap().into_bytes()
            ),
            Err(e) => e
        }
    }
}
