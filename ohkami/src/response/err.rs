use super::{header::ResponseHeaders, status::ErrStatus};
use crate::utils::string::string;

pub struct ErrResponse(
    pub(crate) String
);
