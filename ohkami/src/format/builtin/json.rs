use crate::{FromRequest, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};

#[cfg(all(debug_assertions, feature="openapi"))]
use crate::openapi;


pub struct JSON<T>(pub T);

#[cfg(all(debug_assertions, feature="openapi"))]
mod bound {use super::*;
    pub trait Incoming<'req>: Deserialize<'req> + openapi::support::Schema {}
    impl<'req, T> Incoming<'req> for T where T: Deserialize<'req> + openapi::support::Schema {}

    pub trait Outgoing: Serialize + openapi::support::Schema {}
    impl<T> Outgoing for T where T: Serialize + openapi::support::Schema {}
}
#[cfg(not(all(debug_assertions, feature="openapi")))]
mod bound {use super::*;
    pub trait Incoming<'req>: Deserialize<'req> {}
    impl<'req, T> Incoming<'req> for T where T: Deserialize<'req> {}

    pub trait Outgoing: Serialize {}
    impl<T> Outgoing for T where T: Serialize {}
}

impl<'req, T: bound::Incoming<'req>> FromRequest<'req> for JSON<T> {
    type Error = Response;

    #[inline(always)]
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if req.headers.ContentType()? != "application/json" {
            return None
        }
        serde_json::from_slice(req.payload()?)
            .map_err(super::reject)
            .map(Self).into()
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_input() -> Option<openapi::Input> {
        Some(openapi::Input::Body(openapi::RequestBody::new(
            "application/json",
            T::schema()
        )))
    }
}

impl<T: bound::Outgoing> IntoResponse for JSON<T> {
    #[inline(always)]
    fn into_response(self) -> Response {
        Response::OK().with_json(self.0)
    }

    #[cfg(all(debug_assertions, feature="openapi"))]
    fn openapi_responses() -> openapi::Responses {
        openapi::Responses::new(200, openapi::Response::when("OK")
            .content("application/json", T::schema())
        )
    }
}
