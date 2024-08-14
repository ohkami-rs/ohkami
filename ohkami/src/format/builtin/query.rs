use crate::{Response, FromRequest};
use serde::Deserialize;


pub struct Query<Schema>(pub Schema);

impl<'req, S: Deserialize<'req>> FromRequest<'req> for Query<S> {
    type Error = Response;

    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        req.query.as_ref()?.parse()
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
            .and_then(super::super::validated)
            .map(Query).into()
    }
}
