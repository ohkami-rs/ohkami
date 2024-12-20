use crate::{FromRequest, IntoResponse, Request, Response};

#[cfg(all(debug_assertions, feature="openapi"))]
use crate::openapi;


pub struct Text<T>(pub T);

impl<'req, T: From<&'req str>> FromRequest<'req> for Text<T> {
    type Error = Response;
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if !req.headers.ContentType()?.starts_with("text/plain") {
            return None
        }
        std::str::from_utf8(req.payload()?)
            .map_err(super::reject)
            .map(|s| Self(T::from(s))).into()
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_input() -> Option<openapi::Input> {
        Some(openapi::Input::Body(openapi::RequestBody::new(
            "text/plain",
            openapi::Schema::string()
        )))
    }
}

impl<T: Into<std::borrow::Cow<'static, str>>> IntoResponse for Text<T> {
    fn into_response(self) -> Response {
        Response::OK().with_text(self.0)
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new(200, openapi::Response::when("OK")
            .content("text/html", openapi::Schema::string())
        )
    }
}
