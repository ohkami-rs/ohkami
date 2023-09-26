mod from_request; pub use from_request::*;
mod parse;
mod parse_payload; pub use parse_payload::*;

#[cfg(test)] mod _parse_test;

use std::{borrow::Cow};
use percent_encoding::{percent_decode};
use crate::{
    __dep__,
    layer0_lib::{List, Method, BufRange, Buffer, ContentType}
};


pub(crate) const QUERIES_LIMIT: usize = 4;
pub(crate) const HEADERS_LIMIT: usize = 32;

pub struct Request {
    pub(crate) buffer: Buffer,
    method:  Method,
    path:    BufRange,
    queries: List<(BufRange, BufRange), QUERIES_LIMIT>,
    headers: List<(BufRange, BufRange), HEADERS_LIMIT>,
    payload: Option<(ContentType, BufRange)>,
}

impl Request {
    pub(crate) async fn new(stream: &mut __dep__::TcpStream) -> Self {
        let buffer = Buffer::new(stream).await;
        parse::parse(buffer)
    }
}

impl Request {
    #[inline] pub fn method(&self) -> Method {
        self.method
    }
    #[inline] pub fn path(&self) -> &str {
        unsafe {std::mem::transmute(
            &*(percent_decode(&self.buffer[&self.path]).decode_utf8_lossy())
        )}
    }
    #[inline] pub fn query<Value: FromBuffer>(&self, key: &str) -> Option<Result<Value, Cow<'static, str>>> {
        for (key_range, value_range) in self.queries.iter() {
            if key.eq_ignore_ascii_case(&percent_decode(&self.buffer[key_range]).decode_utf8_lossy()) {
                return Some(Value::parse((&percent_decode(&self.buffer[value_range]).decode_utf8_lossy()).as_bytes()))
            }
        }
        None
    }
    #[inline] pub fn header(&self, key: &str) -> Option<&str> {
        for (key_range, value_range) in self.headers.iter() {
            if key.as_bytes().eq_ignore_ascii_case(&self.buffer[key_range]) {
                return Some(&self.buffer.read_str(value_range))
            }
        }
        None
    }
    #[inline] pub fn payload(&self) -> Option<(&ContentType, &[u8])> {
        let (content_type, body_range) = (&self.payload).as_ref()?;
        Some((
            content_type,
            &self.buffer[body_range],
        ))
    }
}

impl Request {
    #[inline(always)] pub(crate) fn path_bytes(&self) -> &[u8] {
        &self.buffer[&self.path]
    }
}

const _: () = {
    impl std::fmt::Debug for Request {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let queires = {
                let List { list, next } = &self.queries;
                list[..*next].into_iter()
                    .map(|cell| {
                        let (k, v) = unsafe {cell.assume_init_ref()};
                        format!("{} = {}",
                            percent_decode(&self.buffer[k]).decode_utf8_lossy(),
                            percent_decode(&self.buffer[v]).decode_utf8_lossy(),
                        )
                    })
            }.collect::<Vec<_>>();

            let headers = {
                let List { list, next } = &self.headers;
                list[..*next].into_iter()
                    .map(|cell| {
                        let (k, v) = unsafe {cell.assume_init_ref()};
                        format!("{}: {}",
                            self.buffer.read_str(k),
                            self.buffer.read_str(v),
                        )
                    })
            }.collect::<Vec<_>>();

            if let Some((_, payload)) = self.payload() {
                f.debug_struct("Request")
                    .field("method",  &self.method)
                    .field("path",    &self.path())
                    .field("queries", &queires)
                    .field("headers", &headers)
                    .field("payload", &String::from_utf8_lossy(payload))
                    .finish()

            } else {
                f.debug_struct("Request")
                    .field("method",  &self.method)
                    .field("path",    &self.path())
                    .field("queries", &queires)
                    .field("headers", &headers)
                    .finish()
            }
        }
    }
};




#[cfg(test)]
struct DebugRequest {
    method: Method,
    path: &'static str,
    queries: &'static [(&'static str, &'static str)],
    headers: &'static [(&'static str, &'static str)],
    payload: Option<(ContentType, &'static str)>,
}
#[cfg(test)]
const _: () = {
    impl DebugRequest {
        pub(crate) fn assert_parsed_from(self, req_str: &'static str) {
            let DebugRequest { method, path, queries, headers, payload } = self;
            let req = parse::parse(Buffer::from_raw_str(req_str));

            assert_eq!(req.method(), method);
            assert_eq!(req.path(), path);
            assert_eq!(req.payload().map(|(ct, s)| (ct.clone(), std::str::from_utf8(s).unwrap())), payload);
            for (k, v) in queries {
                assert_eq!(req.query::<String>(k), Some(Ok((*v).to_owned())))
            }
            for (k, v) in headers {
                assert_eq!(req.header(k), Some(*v))
            }
        }
    }
};
