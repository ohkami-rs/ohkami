pub mod from_request;
pub mod parse;

use std::ops::Index;
type BufRange = std::ops::Range<usize>;


pub(crate) const REQUEST_BUFFER_SIZE: usize = 1024;
pub(crate) const QUERY_PARAMS_LIMIT : usize = 4;
pub(crate) const PATH_PARAMS_LIMIT  : usize = 2;
pub(crate) const HEADERS_LIMIT      : usize = 32;


pub(crate) struct PathParams {
    params: [Option<BufRange>; PATH_PARAMS_LIMIT],
    next:   u8,
} impl PathParams {
    #[inline(always)] pub(crate) fn new() -> Self {
        Self {
            params: [None, None],
            next:   0,
        }
    }
    #[inline] pub(crate) fn push(&mut self, param: BufRange) {
        if self.next == PATH_PARAMS_LIMIT as u8 {
            tracing::error!("ohkami can't handle more than {PATH_PARAMS_LIMIT} path params")
        } else {
            self.params[self.next as usize].replace(param);
            self.next += 1
        }
    }

    #[inline] pub fn next<'req>(&mut self, request: &'req Request) -> Option<&'req str> {
        let path = request.path();
        match self.params[0].take() {
            Some(range) => Some(&path[range]),
            None => match self.params[1].take() {
                Some(range) => Some(&path[range]),
                None => None,
            }
        }
    }
}


pub struct Request {
    buffer:  Buffer,
    method:  Method,
    path:    BufRange,
    queries: QueryParams,
    headers: Headers,
    body:    Option<BufRange>,
} impl Request {
    #[inline(always)] pub fn path(&self) -> &str {
        &self.buffer[&self.path]
    }
    #[inline(always)] pub fn method(&self) -> &Method {
        &self.method
    }
    #[inline] pub fn query(&self, key: &str) -> Option<&str> {
        let QueryParams { params, next } = &self.queries;
        for k_v in &params[..*next as usize] {
            let (k, v) = k_v.as_ref().unwrap();
            if &self.buffer[k] == key {
                return Some(&self.buffer[v])
            }
        }
        None
    }
    #[inline] pub fn header(&self, key: &str) -> Option<&str> {
        let Headers { headers, next } = &self.headers;
        for k_v in &headers[..*next as usize] {
            let (k, v) = k_v.as_ref().unwrap();
            if &self.buffer[k] == key {
                return Some(&self.buffer[v])
            }
        }
        None
    }
    #[inline] pub fn body(&self) -> Option<&str> {
        Some(&self.buffer[(&self.body).as_ref()?])
    }
}

pub(crate) struct Buffer(
    [u8; REQUEST_BUFFER_SIZE]
); const _: () = {
    impl Index<BufRange> for Buffer {
        type Output = str;
        fn index(&self, range: BufRange) -> &Self::Output {
            unsafe {std::str::from_utf8_unchecked(
                &self.0[range]
            )}
        }
    }
    impl<'r> Index<&'r BufRange> for Buffer {
        type Output = str;
        fn index(&self, range: &'r BufRange) -> &Self::Output {
            unsafe {std::str::from_utf8_unchecked(
                &self.0[range.start..range.end]
            )}
        }
    }
};


pub(crate) enum Method {
    GET, POST, PATCH, DELETE,
} impl Method {
    #[inline] fn parse_bytes(bytes: &[u8]) -> Self {
        match bytes {
            b"GET" => Self::GET,
            b"POST" => Self::POST,
            b"PATCH" => Self::PATCH,
            b"DELETE" => Self::DELETE,
            _ => panic!("unknown method: `{}`", unsafe {std::str::from_utf8_unchecked(bytes)})
        }
    }
}

struct QueryParams {
    params: [Option<(BufRange, BufRange)>; QUERY_PARAMS_LIMIT],
    next:   u8,
} impl QueryParams {
    #[inline] fn new() -> Self {
        Self {
            params: [None, None, None, None],
            next:   0,
        }
    }
    #[inline] fn push(&mut self, key: BufRange, value: BufRange) {
        if self.next == QUERY_PARAMS_LIMIT as u8 {
            panic!("ohkami can't handle more than {QUERY_PARAMS_LIMIT} query parameters")
        } else {
            self.params[self.next as usize].replace((key, value));
            self.next += 1
        }
    }
}

struct Headers {
    headers: [Option<(BufRange, BufRange)>; HEADERS_LIMIT],
    next:    u8,
} impl Headers {
    #[inline] fn new() -> Self {
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
    #[inline] fn append(&mut self, key: BufRange, value: BufRange) {
        if self.next == HEADERS_LIMIT as u8 {
            panic!("ohkami can't handle more than {HEADERS_LIMIT} request headers")
        } else {
            self.headers[self.next as usize].replace((key, value));
            self.next += 1
        }
    }
}
