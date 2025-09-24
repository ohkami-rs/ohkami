use super::super::bound::{self, Incoming, Outgoing};
use super::{FromContent, IntoContent};
use ohkami_lib::serde_urlencoded;
use std::borrow::Cow;

#[cfg(feature = "openapi")]
use crate::openapi;

/// # URL encoded format
///
/// When `openapi` feature is activated, schema bound additionally
/// requires `openapi::Schema`.
///
/// ## Request
///
/// - content type: `application/x-www-form-urlencoded`
/// - schema bound: `Deserialize<'_>`
///
/// ### example
///
/// ```
/// # enum MyError {}
/// use ohkami::claw::content::UrlEncoded;
/// use ohkami::serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct CreateUserRequest<'req> {
///     name: &'req str,
///     age: Option<u8>,
/// }
///
/// async fn create_user(
///     UrlEncoded(body): UrlEncoded<CreateUserRequest<'_>>,
/// ) -> Result<(), MyError> {
///     todo!()
/// }
/// ```
///
/// ## Response
///
/// - content type: `application/x-www-form-urlencoded`
/// - schema bound: `Serialize`
///
/// ### example
///
/// ```
/// # enum MyError {}
/// use ohkami::claw::content::UrlEncoded;
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
/// ) -> Result<UrlEncoded<User>, MyError> {
///     todo!()
/// }
/// ```
pub struct UrlEncoded<T: bound::Schema>(pub T);

impl<'req, T: Incoming<'req>> FromContent<'req> for UrlEncoded<T> {
    const MIME_TYPE: &'static str = "application/x-www-form-urlencoded";

    fn from_content(body: &'req [u8]) -> Result<Self, impl std::fmt::Display> {
        serde_urlencoded::from_bytes(body).map(UrlEncoded)
    }

    #[cfg(feature = "openapi")]
    fn openapi_requestbody() -> impl Into<openapi::schema::SchemaRef> {
        T::schema()
    }
}

impl<T: Outgoing> IntoContent for UrlEncoded<T> {
    const CONTENT_TYPE: &'static str = "application/x-www-form-urlencoded";

    fn into_content(self) -> Result<Cow<'static, [u8]>, impl std::fmt::Display> {
        serde_urlencoded::to_string(&self.0).map(|s| Cow::Owned(s.into_bytes()))
    }

    #[cfg(feature = "openapi")]
    fn openapi_responsebody() -> impl Into<openapi::schema::SchemaRef> {
        T::schema()
    }
}
