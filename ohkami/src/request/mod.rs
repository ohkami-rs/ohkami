pub mod from_request;
pub mod parse;


compile_error!(r"
- PathParams は HandleFunc と一緒に見つかる特別扱い枠とする
- IntoHandleFunc の実装でもあらゆる impl FromRequest な型と同列に扱う
");
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
}


pub(crate) const REQUEST_BUFFER_SIZE: usize = 1024;
pub(crate) const QUERY_PARAMS_LIMIT : usize = 4;
pub(crate) const HEADERS_LIMIT      : usize = 32;

pub struct Request<'buf> {
    pub(crate) method:       &'buf str,
    pub(crate) path:         &'buf str,
    pub(crate) query_params: QueryParams<'buf>,
    pub(crate) headers:      Headers<'buf>,
    pub(crate) body:         Option<&'buf str>,
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
    #[inline] pub(crate) fn push(&mut self, key: &'buf str, value: &'buf str) {
        if self.next == QUERY_PARAMS_LIMIT as u8 {
            tracing::error!("ohkami can't handle more than {QUERY_PARAMS_LIMIT} query parameters")
        } else {
            self.params[self.next as usize].replace((key, value));
            self.next += 1
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
    #[inline] pub(crate) fn append(&mut self, key: &'buf str, value: &'buf str) {
        if self.next == HEADERS_LIMIT as u8 {
            tracing::error!("ohkami can't handle more than {HEADERS_LIMIT} request headers")
        } else {
            self.headers[self.next as usize].replace((key, value));
            self.next += 1
        }
    }
}
