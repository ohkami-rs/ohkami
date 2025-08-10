use super::{FromContent, IntoContent};
use std::borrow::Cow;

#[cfg(feature="openapi")]
use crate::openapi;

/// # plain text format
/// 
/// ## Request
/// 
/// - content type: `text/html; charset=UTF-8`
/// - schema bound: `From<&'_ str>`
/// 
/// ### example
/// 
/// ```
/// use ohkami::claw::content::Text;
/// 
/// async fn accept_text(
///     Text(text): Text<&str>,
/// ) {
///     println!("got plain text request: {text}");
/// }
/// ```
/// 
/// ## Response
/// 
/// - content type: `text/html; charset=UTF-8`
/// - schema bound: `Into<Cow<'static, str>>`
/// 
/// ### note
/// 
/// For `&'static str`, `String` and `Cow<'static, str>`, this is
/// useless because they can be directly available as plain text
/// response.
/// 
/// ### example
/// 
/// ```
/// use ohkami::claw::content::Text;
/// 
/// async fn handler() -> Text<&'static str> {
///     Text(r#"
///         <html>
///             <head>
///                 <title>Sample Document</title>
///             </head>
///             <body>
///                 <h1>Sample Document</h1>
///             </body>
///         </html>
///     "#)
/// }
/// ```
pub struct Text<T>(pub T);

impl<'req, T: From<&'req str>> FromContent<'req> for Text<T> {
    const MIME_TYPE: &'static str = "text/plain";

    fn from_content(body: &'req [u8]) -> Result<Self, impl std::fmt::Display> {
        std::str::from_utf8(body).map(|s| Text(s.into()))
    }

    #[cfg(feature="openapi")]
    fn openapi_requestbody() -> impl Into<openapi::schema::SchemaRef> {
        openapi::string()
    }
}

impl<T: Into<Cow<'static, str>>> IntoContent for Text<T> {
    const CONTENT_TYPE: &'static str = "text/plain; charset=UTF-8";

    fn into_content(self) -> Result<Cow<'static, [u8]>, impl std::fmt::Display> {
        Result::<_, std::convert::Infallible>::Ok(match self.0.into() {
            Cow::Owned(s) => Cow::Owned(s.into_bytes()),
            Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
        })
    }

    #[cfg(feature="openapi")]
    fn openapi_responsebody() -> impl Into<openapi::schema::SchemaRef> {
        openapi::string()
    }
}
