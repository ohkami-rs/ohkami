use crate::IntoBody;

#[cfg(all(debug_assertions, feature="openapi"))]
use crate::openapi;


pub struct HTML<T = String>(pub T);

impl<T: Into<std::borrow::Cow<'static, str>>> IntoBody for HTML<T> {
    const MIME_TYPE: &'static str = "text/html";
    fn into_body(self) -> Result<Vec<u8>, impl std::fmt::Display> {
        let cow: std::borrow::Cow<'static, str> = self.0.into();
        Ok::<_, std::convert::Infallible>(cow.into_owned().into_bytes())
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_responsebody() -> impl Into<openapi::schema::SchemaRef> {
        openapi::Schema::string()
    }
}
