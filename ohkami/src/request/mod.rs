mod from_request;
mod parse;

pub(crate) const REQUEST_BUFFER_SIZE: usize = 1024;
pub(crate) const PATH_PARAMS_LIMIT  : usize = 2;
pub(crate) const QUERY_PARAMS_LIMIT : usize = 4;
pub(crate) const HEADERS_LIMIT      : usize = 32;

pub struct Request<'buf> {
    buffer: [u8; REQUEST_BUFFER_SIZE],
    pub(crate) path_params:  PathParams<'buf>,
    pub(crate) query_params: QueryParams<'buf>,
    pub(crate) headers:      Headers<'buf>,
    pub(crate) body:         Option<&'buf str>,
}

pub(crate) struct PathParams<'buf> {
    params: [Option<&'buf str>; PATH_PARAMS_LIMIT],
    next:   u8,
} impl<'buf> PathParams<'buf> {
    #[inline] pub(crate) fn new() -> Self {
        Self {
            params: [None, None],
            next:   0,
        }
    }
}

pub(crate) struct QueryParams<'buf> {
    params: [Option<(&'buf str, &'buf str)>; QUERY_PARAMS_LIMIT],
    next:   u8,
} impl<'buf> QueryParams<'buf> {
    #[inline] pub(crate) fn new() -> Self {
        Self {
            params: [None, None, None, None],
            next:   0,
        }
    }
}

pub(crate) struct Headers<'buf> {
    headers: [Option<(&'buf str, &'buf str)>; HEADERS_LIMIT],
    next:    u8,
} impl<'buf> Headers<'buf> {
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
