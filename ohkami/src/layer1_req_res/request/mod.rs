mod parse;

use crate::{
    __dep__, Error,
    layer0_lib::{List, Method, BufRange, Buffer, BUFFER_SIZE, ContentType}
};

pub(crate) const QUERIES_LIMIT: usize = 4;
pub(crate) const HEADERS_LIMIT: usize = 32;

pub struct Request {
    buffer:  Buffer,
    method:  Method,
    path:    BufRange,
    queries: List<(BufRange, BufRange), QUERIES_LIMIT>,
    headers: List<(BufRange, BufRange), HEADERS_LIMIT>,
    body:    Option<(ContentType, BufRange)>,
}

impl Request {
    pub(crate) async fn new(stream: &mut __dep__::TcpStream) -> Result<Self, Error> {
        let buffer = Buffer::new(stream).await?;
        Ok(parse::parse(buffer))
    }
}

impl Request {
    pub fn method(&self) -> Method {
        self.method
    }
    pub fn path(&self) -> &str {
        &self.buffer.read_str(&self.path)
    }
    pub fn query(&self, key: &str) -> Option<&str> {
        let List { list, next } = &self.queries;
        for query in &list[..*next] {
            let (key_range, value_range) = unsafe {query.assume_init_ref()};
            if &self.buffer[key_range] == key.as_bytes() {
                return Some(&self.buffer.read_str(value_range))
            }
        }
        None
    }
    pub fn header(&self, key: &str) -> Option<&str> {
        let List { list, next } = &self.headers;
        for header in &list[..*next] {
            let (key_range, value_range) = unsafe {header.assume_init_ref()};
            if &self.buffer[key_range] == key.as_bytes() {
                return Some(&self.buffer.read_str(value_range))
            }
        }
        None
    }
    pub fn body(&self) -> Option<(&ContentType, &str)> {
        let (content_type, body_range) = (&self.body).as_ref()?;
        Some((
            content_type,
            &self.buffer.read_str(body_range),
        ))
    }
}


#[cfg(test)]
struct DebugRequest {
    method: Method,
    path: &'static str,
    queries: &'static [(&'static str, &'static str)],
    headers: &'static [(&'static str, &'static str)],
    body: Option<(ContentType, &'static str)>,
}
#[cfg(test)]
const _: () = {
    impl DebugRequest {
        pub(crate) fn assert_parsed_from(self, req_str: &'static str) {
            let DebugRequest { method, path, queries, headers, body } = self;
            let req = parse::parse(Buffer::from_raw_str(req_str));

            assert_eq!(req.method(), method);
            assert_eq!(req.path(), path);
            assert_eq!(req.body().map(|(ct, s)| (ct.clone(), s)), body);
            for (k, v) in queries {
                assert_eq!(req.query(k), Some(*v))
            }
            for (k, v) in headers {
                assert_eq!(req.header(k), Some(*v))
            }
        }
    }
};
