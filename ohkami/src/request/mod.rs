mod from_request;
pub(crate) mod parse;

use std::ops::Range;

pub(crate) const REQUEST_BUFFER_SIZE: usize = 1024;
pub(crate) const PATH_PARAMS_LIMIT  : usize = 2;
pub(crate) const QUERY_PARAMS_LIMIT : usize = 4;
pub(crate) const HEADERS_LIMIT      : usize = 32;

pub struct Request {
    buffer: [u8; REQUEST_BUFFER_SIZE],
    pub(crate) path_params:  PathParams,
    pub(crate) query_params: QueryParams,
    pub(crate) headers:      Headers,
    pub(crate) body:         Option<Range<usize>>,
}

pub(crate) struct PathParams {
    params: [Option<Range<usize>>; PATH_PARAMS_LIMIT],
    next:   u8,
} impl PathParams {
    #[inline] pub(crate) fn new() -> Self {
        Self {
            params: [None, None],
            next:   0,
        }
    }
}

pub(crate) struct QueryParams {
    params: [Option<(Range<usize>, Range<usize>)>; QUERY_PARAMS_LIMIT],
    next:   u8,
} impl QueryParams {
    #[inline] pub(crate) fn new() -> Self {
        Self {
            params: [None, None, None, None],
            next:   0,
        }
    }
}

pub(crate) struct Headers {
    headers: [Option<(Range<usize>, Range<usize>)>; HEADERS_LIMIT],
    next:    u8,
} impl Headers {
    #[inline] pub(crate) fn new() -> Self {
        Self {
            next:    0,
            headers: [
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None,
            ],
        }
    }
}
