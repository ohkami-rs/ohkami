use crate::FromBody;
use super::bound::{self, Incoming};
use ohkami_lib::serde_multipart;

#[cfg(feature="openapi")]
use crate::openapi;

/// # multipart/form-data format
/// 
/// When `openapi` feature is activated, schema bound additionally
/// requires `openapi::Schema`.
/// 
/// ## Request
/// 
/// - content type: `multipart/form-data`
/// - schema bound: `Deserialize<'_>`
/// 
/// ### example
/// 
/// ```
/// # enum MyError {}
/// use ohkami::format::{Multipart, File};
/// use ohkami::serde::Deserialize;
/// 
/// #[derive(Deserialize)]
/// struct SignUpForm<'req> {
///     #[serde(rename = "user-name")]
///     user_name:  &'req str,
/// 
///     password:   &'req str,
///     
///     #[serde(rename = "user-icon")]
///     user_icon:  Option<File<'req>>,
/// 
///     #[serde(rename = "pet-photos")]
///     pet_photos: Vec<File<'req>>,
/// }
/// 
/// async fn sign_up(
///     Multipart(form): Multipart<SignUpForm<'_>>,
/// ) -> Result<(), MyError> {
///     todo!()
/// }
/// ```
/// 
/// ## Response
/// 
/// not supported
pub use ohkami_lib::serde_multipart::File;

pub struct Multipart<T: bound::Schema>(pub T);

impl<'req, T: Incoming<'req>> FromBody<'req> for Multipart<T> {
    const MIME_TYPE: &'static str = "multipart/form-data";

    fn from_body(body: &'req [u8]) -> Result<Self, impl std::fmt::Display> {
        serde_multipart::from_bytes(body).map(Multipart)
    }

    #[cfg(feature="openapi")]
    fn openapi_requestbody() -> impl Into<openapi::schema::SchemaRef> {
        T::schema()
    }
}
