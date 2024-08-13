use crate::{Response, FromRequest};
use serde::Deserialize;


pub struct Query<Schema>(pub Schema);

impl<'req, S: Deserialize<'req>> FromRequest<'req> for Query<S> {
    type Error = Response;

    #[cfg(not(feature="nightly"))]
    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        req.query.as_ref()?.parse()
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
            .map(Query).into()
    }

    #[cfg(feature="nightly")]
    default fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        req.query.as_ref()?.parse()
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
            .map(Query).into()
    }
}

#[cfg(feature="nightly")]
impl<'req, S: Deserialize<'req> + super::Schema> FromRequest<'req> for Query<S> {
    type Error;

    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        match req.query.as_ref()?.parse::<S>()
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
        {
            Err(e)   => Some(Err(e)),
            Ok(this) => match this.valid()
                .map_err(|e| Response::BadRequest().with_text(e.to_string()))
            {
                Err(e) => Some(Err(e)),
                Ok(_)  => Some(Ok(Query(this)))
            }
        }
    }
}
