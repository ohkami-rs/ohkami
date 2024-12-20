use crate::{IntoResponse, Response};

#[cfg(all(debug_assertions, feature="openapi"))]
use crate::openapi;


pub struct HTML<T = String>(pub T);

impl<T: Into<std::borrow::Cow<'static, str>>> IntoResponse for HTML<T> {
    fn into_response(self) -> Response {
        Response::OK().with_html(self.0)
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new(200, openapi::Response::when("OK")
            .content("text/html", openapi::Schema::string())
        )
    }
}
