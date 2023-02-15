use crate::{testing::Method, components::headers::RequestHeaders, router::path::Path};

pub mod query;
pub mod path;
pub mod body;

pub trait FromRequest {

}


pub(crate) struct RawRequest<'buf> {
    pub(crate) method:  Method,
    pub(crate) path:    Path<'buf>,
    pub(crate) queries: &'buf str,
    pub(crate) headers: RequestHeaders<'buf>,
}
