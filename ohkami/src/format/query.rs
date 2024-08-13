use crate::{Response, FromRequest};
use serde::Deserialize;


pub struct Query<Schema>(pub Schema);

impl<'req, S: Deserialize<'req>> FromRequest<'req> for Query<S> {
    type Error = Response;

    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        Some(
            req.query.as_ref()?.parse()
            .map(Query)
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
        )
    }
}
