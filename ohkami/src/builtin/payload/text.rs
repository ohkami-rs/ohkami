use serde::{Serialize, Deserialize};
use ohkami_lib::serde_utf8;
use crate::typed::PayloadType;

/// Builtin `PayloadType` for `text/plain` payloads.
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```
/// use ohkami::typed::Payload;
/// use ohkami::builtin::payload::Text; //
/// 
/// #[Payload(Text/SD)]
/// struct TextMessage(String);
/// 
/// async fn echo_text(
///     msg: TextMessage
/// ) -> TextMessage {
///     msg
/// }
/// ```
/// ---
pub struct Text;
impl PayloadType for Text {
    const MIME_TYPE: &'static str = "text/plain";
    const CONTENT_TYPE: &'static str = "text/plain; charset=UTF-8";

    #[inline]
    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
        let str = std::str::from_utf8(bytes).map_err(
            |e| serde::de::Error::custom(format!("input is not UTF-8: {e}"))
        )?;
        serde_utf8::from_str(str)
    }

    #[inline(always)]
    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
        serde_utf8::to_string(value).map(String::into_bytes)
    }
}

/// Builtin `PayloadType` for `text/html` payloads.
/// 
/// _**note**_ : This doesn't check the content is valid HTML,
/// just assume the content is handled as a `text/html` payload.
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```
/// use ohkami::typed::Payload;
/// use ohkami::builtin::payload::HTML; //
/// # use ohkami::serde::{Serialize, ser};
/// 
/// #[Payload(HTML)]
/// struct PetListTemplate {
///     pets: Vec<Pet>,
/// }
/// 
/// impl Serialize for PetListTemplate {
///     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
///     where S: ohkami::serde::Serializer {
///           let html = self.render().map_err(|e| ser::Error::custom(e.to_string()))?;
///             serializer.serialize_str(&html)
///       }
/// }
/// 
/// # struct Pet {
/// #     name: String,
/// #     age:  Option<usize>,
/// # }
/// # 
/// # impl PetListTemplate {
/// #     fn render(&self) -> Result<String, String> {
/// #         todo!()
/// #     }
/// # }
/// 
/// async fn get_pets() -> PetListTemplate {
/// #    async fn get_all_pets() -> Vec<Pet> {
/// #        Vec::new()
/// #    }
///     PetListTemplate {
///         pets: get_all_pets().await,
///     }
/// }
/// ```
/// ---
pub struct HTML;
impl PayloadType for HTML {
    const MIME_TYPE: &'static str = "text/html";
    const CONTENT_TYPE: &'static str = "text/html; charset=UTF-8";

    fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, impl crate::serde::ser::Error> {
        serde_utf8::to_string(value).map(String::into_bytes)
    }

    fn parse<'req, T: Deserialize<'req>>(bytes: &'req [u8]) -> Result<T, impl crate::serde::de::Error> {
        let str = std::str::from_utf8(bytes).map_err(
            |e| serde::de::Error::custom(format!("input is not UTF-8: {e}"))
        )?;
        serde_utf8::from_str(str)
    }
}
