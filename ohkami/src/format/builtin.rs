use crate::{FromRequest, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};


pub struct JSON<Schema>(pub Schema);
impl<'req, S: Deserialize<'req>> FromRequest<'req> for JSON<S> {
    type Error = Response;
    #[inline(always)]
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if req.headers.ContentType()? != "application/json" {
            return None
        }
        serde_json::from_slice(req.payload.as_deref()?)
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
            .map(Self).into()
    }
}
impl<S: Serialize> IntoResponse for JSON<S> {
    #[inline(always)]
    fn into_response(self) -> Response {
        Response::OK().with_json(self.0)
    }
}

pub struct Multipart<Schema>(pub Schema);
impl<'req, S: Deserialize<'req>> FromRequest<'req> for Multipart<S> {
    type Error = Response;
    #[inline]
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if req.headers.ContentType()? != "multipart/form-data" {
            return None
        }
        ohkami_lib::serde_multipart::from_bytes(req.payload.as_deref()?)
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
            .map(Self).into()
    }
}
pub use ohkami_lib::serde_multipart::File;

pub struct URLEncoded<Schema>(pub Schema);
impl<'req, S: Deserialize<'req>> FromRequest<'req> for URLEncoded<S> {
    type Error = Response;
    #[inline]
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if req.headers.ContentType()? != "x-www-form-urlencoded" {
            return None
        }
        ohkami_lib::serde_urlencoded::from_bytes(req.payload.as_deref()?)
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
            .map(Self).into()
    }
}
impl<S: Serialize> IntoResponse for URLEncoded<S> {
    fn into_response(self) -> Response {
        Response::OK().with_payload(
            "x-www-form-urlencoded",
            ohkami_lib::serde_urlencoded::to_string(&self.0).unwrap().into_bytes()
        )
    }
}

pub struct Text<T>(pub T);
impl<'req, T: From<&'req str>> FromRequest<'req> for Text<T> {
    type Error = Response;
    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        if !req.headers.ContentType()?.starts_with("text/plain") {
            return None
        }
        std::str::from_utf8(req.payload.as_deref()?)
            .map_err(|e| Response::BadRequest().with_text(e.to_string()))
            .map(|s| Self(T::from(s))).into()
    }
}
impl<T: Into<std::borrow::Cow<'static, str>>> IntoResponse for Text<T> {
    fn into_response(self) -> Response {
        Response::OK().with_text(self.0)
    }
}

pub struct HTML<T = String>(pub T);
impl<T: Into<std::borrow::Cow<'static, str>>> IntoResponse for HTML<T> {
    fn into_response(self) -> Response {
        Response::OK().with_html(self.0)
    }
}
