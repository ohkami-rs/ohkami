use crate::{Response, FromRequest};
use super::bound::{self, Incoming};

#[cfg(feature="openapi")]
use crate::openapi;


/// # Query parameters
/// 
/// Deserialize query parameters in a request into an instance of
/// schema type `T: Deserialize<'_>`.
/// 
/// When `openapi` feature is activated, schema bound additionally
/// requires `openapi::Schema`.
/// 
/// ### example
/// 
/// ```
/// # enum MyError {}
/// use ohkami::format::{JSON, Query};
/// use ohkami::serde::Deserialize;
/// 
/// #[derive(Deserialize)]
/// struct ListUsersMeta<'req> {
///     name_prefix: Option<&'req str>,
///     min_age: Option<u8>,
///     max_age: Option<u8>,
///     limit: Option<usize>,
/// }
/// # #[derive(ohkami::serde::Serialize)]
/// # struct User {}
/// 
/// async fn list_users(
///     Query(meta): Query<ListUsersMeta<'_>>,
/// ) -> Result<JSON<Vec<User>>, MyError> {
///     todo!()
/// }
/// ```
/// 
/// ### note
/// 
/// When a request doesn't have query parameters, `Option<Query<T>>` in a handler
/// tries to deserialize an *empty query string*, not skip deserializing
/// returning `None`.
/// This may be unexpected behavior and just *`Query<T>` with `Option<_>` fields*
/// is recommended to express *optional query params*.
pub struct Query<T: bound::Schema>(pub T);

impl<'req, T: Incoming<'req>> FromRequest<'req> for Query<T> {
    type Error = Response;

    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        req.query.parse()
            .map_err(super::reject)
            .map(Query).into()
    }

    #[cfg(feature="openapi")]
    fn openapi_inbound() -> openapi::Inbound {
        let Some(schema) = T::schema().into().into_inline() else {
            return openapi::Inbound::None
        };
        openapi::Inbound::Params(
            schema.into_properties().into_iter().map(|(name, schema, required)|
                if required {
                    openapi::Parameter::in_query(name, schema)
                } else {
                    openapi::Parameter::maybe_in_query(name, schema)
                }
            ).collect()
        )
    }
}
