use serde::Serialize;
use crate::{
    utils::{map::RANGE_MAP_SIZE, buffer::Buffer}
};
pub use crate::components::method::Method;

#[allow(unused)]
pub struct Request {
    method: Method,
    uri:    &'static str,
    query:  [Option<(&'static str, &'static str)>; RANGE_MAP_SIZE],
    body:   Option<String>,
} impl Request {
    pub fn new(method: Method, uri: &'static str) -> Self {
        Self {
            method,
            uri,
            query: [None, None, None, None],
            body:  None,
        }
    }
    pub fn query(mut self, query: (&'static str, &'static str)) -> Self {
        let index = 'index: {
            for (i, q) in self.query.iter().enumerate() {
                if q.is_none() {break 'index i}
            }
            panic!("Current ohkami can't handle more than {RANGE_MAP_SIZE} query params");
        };
        self.query[index] = Some(query);
        self
    }
    pub fn body<S: Serialize>(mut self, body: S) -> Self {
        let body = serde_json::to_string(&body).expect("can't serialize given body as a JSON");
        self.body = Some(body);
        self
    }

    #[allow(unused)]
    pub(crate) async fn into_request_buffer(&self) -> Buffer {
        let request_uri = {
            let mut uri = self.uri.to_owned();
            if self.query[0].is_some() {
                for (i, query) in self.query.iter().enumerate() {
                    match query {
                        None => break,
                        Some((key, value)) => {
                            uri.push(if i==0 {'?'} else {'&'});
                            uri += key;
                            uri.push('=');
                            uri += value;
                        },
                    }
                }
            };
            uri
        };
        let request_str = {
            let mut raw_request = format!(
"{} {} HTTP/1.1
",
                self.method,
                request_uri,
            );
            if let Some(body) = &self.body {
                raw_request.push('\n');
                raw_request += &body
            }
            raw_request
        };
        Buffer::from_http_request_str(request_str).await
    }
}