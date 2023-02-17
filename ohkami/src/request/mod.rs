use crate::{components::headers::RequestHeaders};

pub mod query;
pub mod path;
pub mod body;

pub trait FromRequest {
    fn from_request<'buf>(request: &Request<'buf>) -> Self;
}

pub(crate) struct Request<'buf> {
    pub path:    Path<'buf>,
    pub queries: &'buf str,
    pub headers: RequestHeaders<'buf>,
    pub body:    Option<&'buf str>,
}
