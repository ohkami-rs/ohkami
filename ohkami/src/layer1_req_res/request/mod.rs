mod parse;

use crate::{
    __feature__::{self, StreamReader}, Error,
    layer0_lib::{List, Method, BufRange, Buffer, BUFFER_SIZE}
};


pub(crate) const QUERIES_LIMIT: usize = 4;
pub(crate) const HEADERS_LIMIT: usize = 32;

pub struct Request {
    buffer:  Buffer,
    method:  Method,
    path:    BufRange,
    queries: List<(BufRange, BufRange), QUERIES_LIMIT>,
    headers: List<(BufRange, BufRange), HEADERS_LIMIT>,
    body:    Option<BufRange>,
}

impl Request {
    pub(crate) async fn new(stream: &mut __feature__::TcpStream) -> Result<Self, Error> {
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
            let (key_range, value_range) = query.as_ref().unwrap();
            if &self.buffer[key_range] == key.as_bytes() {
                return Some(&self.buffer.read_str(value_range))
            }
        }
        None
    }
    pub fn header(&self, key: &str) -> Option<&str> {
        let List { list, next } = &self.headers;
        for header in &list[..*next] {
            let (key_range, value_range) = header.as_ref().unwrap();
            if &self.buffer[key_range] == key.as_bytes() {
                return Some(&self.buffer.read_str(value_range))
            }
        }
        None
    }
    pub fn body(&self) -> Option<&str> {
        Some(&self.buffer.read_str(
            (&self.body).as_ref()?
        ))
    }
}
