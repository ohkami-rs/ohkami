use serde::{Serialize, Deserialize};
use ohkami_lib::serde_urlencoded;
use crate::typed::PayloadType;


/// Builtin `PayloadType` for `application/x-www-form-urlencoded` payloads.
/// 
/// _**note**_ : \
/// While non encoded value like `ohkami`
/// can be handled as `&'req str`, urlencoded value like
/// `%E3%81%8A%E3%81%8A%E3%81%8B%E3%81%BF` is automatically
/// decoded into `String` (then, it fails deserializing
/// if the corresponded field has type `&str`). \
/// So, if you have a field that's may or may not encoded,
/// `Cow<'req, str>` is the best choice.
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```
/// use ohkami::typed::Payload;
/// use ohkami::builtin::payload::URLEncoded; //
/// # use std::borrow::Cow;
/// 
/// #[Payload(URLEncoded/D)]
/// struct ExampleURLEncoded<'req> {
///     name:    &'req str,
///     profile: Cow<'req, str>,
/// }
/// 
/// 
/// use ohkami::typed::status::OK;
/// 
/// async fn handle_urlencoded(
///     ue: ExampleURLEncoded<'_>
/// ) -> OK {
///     println!(
///         "got example urlencoded: name = {:?}, profile = {:?}",
///         ue.name,
///         ue.profile,
///     );
/// 
///     OK(())
/// }
/// ```
/// ---
pub struct URLEncoded;

impl PayloadType for URLEncoded {
    const MIME_TYPE: &'static str = "application/x-www-form-urlencoded";

    #[inline]
    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
        serde_urlencoded::from_bytes(bytes)
    }

    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
        serde_urlencoded::to_string(value).map(String::into_bytes)
    }
}
