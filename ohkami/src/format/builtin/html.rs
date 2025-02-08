use crate::IntoBody;
use std::borrow::Cow;

#[cfg(feature="openapi")]
use crate::openapi;


pub struct HTML<T = String>(pub T);

impl<T: Into<Cow<'static, str>>> IntoBody for HTML<T> {
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
