use crate::{FromBody, IntoBody};
use super::bound::{self, Incoming, Outgoing};
use std::borrow::Cow;

#[cfg(feature="openapi")]
use crate::openapi;

/// # JSON format
/// 
/// When `openapi` feature is activated, schema bound additionally
/// requires `openapi::Schema`.
/// 
/// ## Request
/// 
/// - content type: `application/json`
/// - schema bound: `Deserialize<'_>`
/// 
/// ### example
/// 
/// ```
/// # enum MyError {}
/// use ohkami::format::JSON;
/// use ohkami::serde::Deserialize;
/// 
/// #[derive(Deserialize)]
/// struct CreateUserRequest<'req> {
///     name: &'req str,
///     age: Option<u8>,
/// }
/// 
/// async fn create_user(
///     JSON(body): JSON<CreateUserRequest<'_>>,
/// ) -> Result<(), MyError> {
///     todo!()
/// }
/// ```
/// 
/// ## Response
/// 
/// - content type: `application/json`
/// - schema bound: `Serialize`
/// 
/// ### example
/// 
/// ```
/// # enum MyError {}
/// use ohkami::format::JSON;
/// use ohkami::serde::Serialize;
/// 
/// #[derive(Serialize)]
/// struct User {
///     name: String,
///     age: Option<u8>,
/// }
/// 
/// async fn get_user(
///     id: &str,
/// ) -> Result<JSON<User>, MyError> {
///     todo!()
/// }
/// ```
pub struct JSON<T: bound::Schema>(pub T);

impl<'req, T: Incoming<'req>> FromBody<'req> for JSON<T> {
    const MIME_TYPE: &'static str = "application/json";

    #[inline]
    fn from_body(body: &'req [u8]) -> Result<Self, impl std::fmt::Display> {
        serde_json::from_slice(body).map(JSON)
    }

    #[cfg(feature="openapi")]
    fn openapi_requestbody() -> impl Into<openapi::schema::SchemaRef> {
        T::schema()
    }
}

impl<T: Outgoing> IntoBody for JSON<T> {
    const CONTENT_TYPE: &'static str = "application/json";

    #[inline]
    fn into_body(self) -> Result<Cow<'static, [u8]>, impl std::fmt::Display> {
        serde_json::to_vec(&self.0).map(Cow::Owned)
    }

    #[cfg(feature="openapi")]
    fn openapi_responsebody() -> impl Into<openapi::schema::SchemaRef> {
        T::schema()
    }
}
