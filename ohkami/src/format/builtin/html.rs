use crate::{IntoResponse, Response};


pub struct HTML<T = String>(pub T);

impl<T: Into<std::borrow::Cow<'static, str>>> IntoResponse for HTML<T> {
    fn into_response(self) -> Response {
        match super::super::validated(self.0) {
            Ok(v)  => Response::OK().with_html(v),
            Err(e) => e,
        }
    }
}
