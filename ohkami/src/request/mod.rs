mod from_request;
pub(crate) mod parse;

use std::ops::Range;

pub(crate) const REQUEST_BUFFER_SIZE: usize = 1024;
pub(crate) const PATH_PARAMS_LIMIT  : usize = 2;
pub(crate) const QUERY_PARAMS_LIMIT : usize = 4;
pub(crate) const HEADERS_LIMIT      : usize = 32;

pub struct Request {
    buffer: [u8; REQUEST_BUFFER_SIZE],
    pub path_params:  [Range<usize>; PATH_PARAMS_LIMIT],
    pub query_params: [(Range<usize>, Range<usize>); QUERY_PARAMS_LIMIT],
    pub headers:      [(Range<usize>, Range<usize>); HEADERS_LIMIT],
    pub body:         Option<Range<usize>>,
}
