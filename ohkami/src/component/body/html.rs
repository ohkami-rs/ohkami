use super::IntoBody;
use std::borrow::Cow;

#[cfg(feature="openapi")]
use crate::openapi;

/// # HTML format
/// 
/// ## Request
/// 
/// not supported
/// 
/// ## Response
/// 
/// - content type: `text/html; charset=UTF-8`
/// - schema bound: `Into<Cow<'static, str>>`
/// 
/// note: This doesn't validate the content to be a valid HTML document,
/// it just sets the content type to `text/html` and returns the content as is.
/// 
/// ### example
/// 
/// ```
/// use ohkami::component::body::Html;
/// 
/// async fn handler() -> Html<&'static str> {
///     Html(r#"
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
pub struct Html<T = String>(pub T);

impl<T: Into<Cow<'static, str>>> IntoBody for Html<T> {
    const CONTENT_TYPE: &'static str = "text/html; charset=UTF-8";

    fn into_body(self) -> Result<Cow<'static, [u8]>, impl std::fmt::Display> {
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
