use crate::utils::string::string;
use super::{header::ResponseHeaders, status::OkStatus};

pub struct OkResponse<T> {
    additional_headers: ResponseHeaders,
    content_type: &'static str,
    status: OkStatus,
    body:   Option<string>,
}
