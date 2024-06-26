use serde::{Serialize, Deserialize};
use ohkami_lib::serde_multipart;
use crate::typed::PayloadType;


/// Builtin `PayloadType` for `multipart/form-data` payloads.
/// 
/// _**note**_ : `Multipart` only supports parding request payload with `Deserialize`.
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```
/// use ohkami::typed::Payload;
/// use ohkami::builtin::payload::Multipart; //
/// 
/// #[Payload(Multipart/D)]
/// struct ExampleForm<'req> {
///     #[serde(rename = "user-name")]
///     user_name: &'req str,
/// 
///     #[serde(rename = "favorite-word")]
///     favorite_word: Option<&'req str>,
/// }
/// 
/// 
/// use ohkami::typed::status::OK;
/// # struct MyAPIError;
/// # impl ohkami::IntoResponse for MyAPIError {
/// #     fn into_response(self) -> ohkami::Response {
/// #         ohkami::Response::InternalServerError()
/// #     }
/// # }
/// 
/// async fn handle_example_form(
///     form: ExampleForm<'_>
/// ) -> Result<OK, MyAPIError> {
///     println!("got example form, user-name = {:?}, favorite-word = {:?}",
///         form.user_name,
///         form.favorite_word,
///     );
/// 
///     Ok(OK(()))
/// }
/// ```
/// ---
/// 
/// <br>
/// 
/// For file uploading, ohkami provides `builtin::utils::File`,
/// that has `filename: &str`, `mimetype: &str`, and `content: &[u8]` of an uploaded files.
/// 
/// <br>
/// 
/// ---
/// *file_upload.rs*
/// ```
/// use ohkami::typed::Payload;
/// use ohkami::builtin::payload::Multipart;
/// use ohkami::builtin::item::File; //
/// 
/// #[Payload(Multipart/D)]
/// struct ExampleForm<'req> {
///     #[serde(rename = "user-name")]
///     user_name: &'req str,
/// 
///     #[serde(rename = "favorite-pic")]
///     favorite_picture: Option<File<'req>>,
/// 
///     #[serde(rename = "pet-pics")]
///     pet_pics: Vec<File<'req>>,
/// }
/// 
/// 
/// use ohkami::typed::status::OK;
/// # struct MyAPIError;
/// # impl ohkami::IntoResponse for MyAPIError {
/// #     fn into_response(self) -> ohkami::Response {
/// #         ohkami::Response::InternalServerError()
/// #     }
/// # }
/// 
/// async fn handle_example_form(
///     form: ExampleForm<'_>
/// ) -> Result<OK, MyAPIError> {
///     println!("got example form: user-name = {:?}, {} pet pics",
///         form.user_name,
///         form.pet_pics.len(),
///     );
/// 
///     Ok(OK(()))
/// }
/// ```
/// ---
pub struct Multipart;
impl PayloadType for Multipart {
    const MIME_TYPE: &'static str = "multipart/form-data";

    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
        serde_multipart::from_bytes(bytes)
    }
    fn bytes<T: Serialize>(_: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
        Err(<std::fmt::Error as crate::serde::ser::Error>::custom(
            "ohkami's builtin `Multipart` payload type doesn't support response use"
        ))
    }
}
