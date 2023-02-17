use super::{header::ResponseHeaders, status::ErrStatus};
use crate::utils::string::string;

pub struct ErrResponse {
    additional_headers: ResponseHeaders,
    status: ErrStatus,
    body:   Option<string>,
}
