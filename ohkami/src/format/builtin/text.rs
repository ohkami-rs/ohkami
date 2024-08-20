use crate::{FromRequest, IntoResponse, Request, Response};


pub struct Text<T>(pub T);

impl<'req, T: From<&'req str>> FromRequest<'req> for Text<T> {
    type Error = Response;
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if !req.headers.ContentType()?.starts_with("text/plain") {
            return None
        }
        std::str::from_utf8(req.payload()?)
            .map_err(super::super::reject)
            .and_then(super::super::validated)
            .map(|s| Self(T::from(s))).into()
    }
}

impl<T: Into<std::borrow::Cow<'static, str>>> IntoResponse for Text<T> {
    fn into_response(self) -> Response {
        match super::super::validated(self.0) {
            Ok(v)  => Response::OK().with_text(v),
            Err(e) => e,
        }
    }
}
