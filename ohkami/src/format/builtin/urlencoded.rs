use crate::{FromRequest, IntoResponse, Request, Response};
use super::bound::{Incoming, Outgoing};

#[cfg(all(debug_assertions, feature="openapi"))]
use crate::openapi;


pub struct URLEncoded<T>(pub T);

impl<'req, T: Incoming<'req>> FromRequest<'req> for URLEncoded<T> {
    type Error = Response;

    #[inline]
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if req.headers.ContentType()? != "application/x-www-form-urlencoded" {
            return None
        }
        ohkami_lib::serde_urlencoded::from_bytes(req.payload()?)
            .map_err(super::reject)
            .map(Self).into()
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_input() -> Option<openapi::Input> {
        Some(openapi::Input::Body(openapi::RequestBody::of(
            "application/x-www-form-urlencoded",
            T::schema()
        )))
    }
}

impl<T: Outgoing> IntoResponse for URLEncoded<T> {
    fn into_response(self) -> Response {
        Response::OK().with_payload("application/x-www-form-urlencoded",
            ohkami_lib::serde_urlencoded::to_string(&self.0).unwrap().into_bytes()
        )
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new(200, openapi::Response::when("OK")
            .content("application/x-www-form-urlencoded", T::schema())
        )
    }
}
