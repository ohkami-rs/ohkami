use crate::{Response, FromRequest};
use super::bound::{self, Incoming};

#[cfg(feature="openapi")]
use crate::openapi;


pub struct Query<T: bound::Schema>(pub T);

impl<'req, T: Incoming<'req>> FromRequest<'req> for Query<T> {
    type Error = Response;

    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        req.query.as_ref()?.parse()
            .map_err(super::reject)
            .map(Query).into()
    }

    #[cfg(feature="openapi")]
    fn openapi_input() -> Option<openapi::request::Input> {
        let schema = T::schema().into().into_inline()?;
        Some(openapi::request::Input::Params(
            schema.into_properties().into_iter().map(|(name, schema)|
                openapi::Parameter::in_query(name, schema)
            ).collect()
        ))
    }
}
