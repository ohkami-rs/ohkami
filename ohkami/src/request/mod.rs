pub mod from_request;
pub mod parse;


pub(crate) const PATH_PARAMS_LIMIT  : usize  = 2;
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
    #[inline] pub(crate) fn push(&mut self, param: &'buf str) {
        if self.next == PATH_PARAMS_LIMIT as u8 {
            tracing::error!("ohkami can't handle more than {PATH_PARAMS_LIMIT} path params")
        } else {
            self.params[self.next as usize].replace(param);
            self.next += 1
        }
    }
}


pub(crate) const REQUEST_BUFFER_SIZE: usize = 1024;
pub(crate) const QUERY_PARAMS_LIMIT : usize = 4;
pub(crate) const HEADERS_LIMIT      : usize = 32;

pub struct Request {
    pub(crate) buffer: [u8; REQUEST_BUFFER_SIZE],
    pub(crate) method:  Method,
    pub(crate) path:    BufRange,
    pub(crate) queries: QueryParams,
    pub(crate) headers: Headers,
    pub(crate) body:    Option<BufRange>,
}
pub(crate) type BufRange = std::ops::Range<usize>;

pub enum Method {
    GET,
    POST,
    PATCH,
    DELETE,
} impl Method {
    pub(crate) fn parse(bytes: &[u8]) -> Self {
        match bytes {
            b"GET" => Self::GET,
            b"POST" => Self::POST,
            b"PATCH" => Self::PATCH,
            b"DELETE" => Self::DELETE,
            _ => panic!("unknown method: {}", unsafe{ std::str::from_utf8_unchecked(bytes) })
        }
    }
}

pub(crate) struct QueryParams {
    params: [Option<(BufRange, BufRange)>; QUERY_PARAMS_LIMIT],
    next:   u8,
} impl QueryParams {
    #[inline] pub(crate) fn new() -> Self {
        Self {
            params: [None, None, None, None],
            next:   0,
        }
    }
    #[inline] pub(crate) fn push(&mut self, key: BufRange, value: BufRange) {
        if self.next == QUERY_PARAMS_LIMIT as u8 {
            tracing::error!("ohkami can't handle more than {QUERY_PARAMS_LIMIT} query parameters")
        } else {
            self.params[self.next as usize].replace((key, value));
            self.next += 1
        }
    }
}

pub(crate) struct Headers {
    headers: [Option<(BufRange, BufRange)>; HEADERS_LIMIT],
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
    #[inline] pub(crate) fn append(&mut self, key: BufRange, value: BufRange) {
        if self.next == HEADERS_LIMIT as u8 {
            tracing::error!("ohkami can't handle more than {HEADERS_LIMIT} request headers")
        } else {
            self.headers[self.next as usize].replace((key, value));
            self.next += 1
        }
    }
}
