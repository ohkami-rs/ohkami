use crate::{Response, FromRequest};
use super::bound::Incoming;

#[cfg(all(debug_assertions, feature="openapi"))]
use crate::openapi;


pub struct Query<Schema>(pub Schema);

impl<'req, T: Incoming<'req>> FromRequest<'req> for Query<T> {
    type Error = Response;

    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        req.query.as_ref()?.parse()
            .map_err(super::reject)
            .map(Query).into()
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_input() -> Option<openapi::Input> {
        Some(openapi::Input::Param(openapi::Parameter::in_query(, schema)))
    }
}
