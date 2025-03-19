#![cfg(debug_assertions)]

//! Ohkami testing tools
//! 
//! <br>
//! 
//! *test_example.rs*
//! ```
//! use ohkami::prelude::*;
//! use ohkami::testing::*;
//! 
//! fn my_ohkami() -> Ohkami {
//!     Ohkami::new(
//!         "/".GET(|| async {
//!             "Hello, ohkami!"
//!         })
//!     )
//! }
//! 
//! #[cfg(test)]
//! #[tokio::test]
//! async fn test_my_ohkami() {
//!     let t = my_ohkami().test();
//! 
//!     let req = TestRequest::GET("/");
//!     let res = t.oneshot(req).await;
//!     assert_eq!(res.status(), Status::OK);
//!     assert_eq!(res.text(), Some("Hello, ohkami!"));
//! }
//! ```

use crate::{Response, Request, Ohkami, Status, Method};
use crate::router::r#final::Router;

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::{pin::Pin, future::Future, format as f};


pub trait Testing {
    fn test(self) -> TestingOhkami;
}

pub struct TestingOhkami(Arc<Router>);

impl Testing for Ohkami {
    fn test(self) -> TestingOhkami {
        let (f, _) = self.into_router().finalize();
        TestingOhkami(Arc::new(f))
    }
}

impl TestingOhkami {
    #[must_use]
    pub fn oneshot(&self, req: TestRequest) -> Oneshot {
        let router = self.0.clone();
        
        let res = async move {
            let mut request = Request::init(#[cfg(feature="__rt_native__")] crate::util::IP_0000);
            let mut request = unsafe {Pin::new_unchecked(&mut request)};
            
            let res = match request.as_mut().read(&mut &req.encode()[..]).await {
                Ok(Some(())) => router.handle(&mut request).await,
                Ok(None) => panic!("No request"),
                Err(res) => res,
            };

            TestResponse::new(res)
        };

        Oneshot(Box::new(res))
    }
}

pub struct Oneshot(
    Box<dyn Future<Output = TestResponse>>
); impl Future for Oneshot {
    type Output = TestResponse;
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        unsafe {self.map_unchecked_mut(|this| this.0.as_mut())}.poll(cx)
    }
}

pub struct TestRequest {
    method:  Method,
    path:    Cow<'static, str>,
    queries: HashMap<Cow<'static, str>, Cow<'static, str>>,
    headers: HashMap<Cow<'static, str>, Cow<'static, str>>,
    content: Option<Cow<'static, [u8]>>,
} impl TestRequest {
    pub(crate) fn encode(self) -> Vec<u8> {
        let Self { method, path, queries, headers, content } = self;

        let queries = queries.into_iter()
            .map(|(k, v)| f!("{k}={v}"))
            .fold(Vec::new(), |mut q, kv| if q.is_empty() {
                q.push(b'?'); q.extend_from_slice(kv.as_bytes()); q
            } else {
                q.push(b'&'); q.extend_from_slice(kv.as_bytes()); q
            });

        let headers = headers.into_iter()
            .map(|(k, v)| f!("{k}: {v}\r\n"))
            .fold(Vec::new(), |mut h, kv| {
                h.extend_from_slice(kv.as_bytes()); h
            });

        [
            method.as_str().as_bytes(), b" ", path.as_bytes(), &queries, b" HTTP/1.1\r\n",
            &headers,
            b"\r\n",
            &content.unwrap_or(Cow::Borrowed(b""))
        ].concat()
    }
}

macro_rules! new_test_request {
    ( $($method:ident)* ) => {$(
        #[allow(non_snake_case)]
        impl TestRequest {
            pub fn $method(path: impl Into<Cow<'static, str>>) -> Self {
                Self {
                    method:  Method::$method,
                    path:    path.into(),
                    queries: HashMap::new(),
                    headers: HashMap::new(),
                    content: None,
                }
            }
        }
    )*};
} new_test_request! {
    GET PUT POST PATCH DELETE HEAD OPTIONS
}

impl TestRequest {
    pub fn query(mut self, key: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) -> Self {
        self.queries.insert(key.into(), value.into());
        self
    }
    pub fn header(mut self, key: impl Into<Cow<'static, str>>, value: impl Into<Cow<'static, str>>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}
impl TestRequest {
    pub fn json(mut self, json: impl serde::Serialize) -> Self {
        let content       = serde_json::to_vec(&json).expect("Failed to serialize json");
        let content_lenth = content.len();

        self.content = Some(Cow::Owned(content));
        self.header("Content-Type", "application/json")
            .header("Content-Length", content_lenth.to_string())
    }
    pub fn json_lit(mut self, json: impl Into<Cow<'static, str>>) -> Self {
        let content: Cow<'static, [u8]> = match json.into() {
            Cow::Borrowed(str) => Cow::Borrowed(str.as_bytes()),
            Cow::Owned(string) => Cow::Owned(string.into_bytes()),
        };
        let content_lenth = content.len();

        self.content = Some(content);
        self.header("Content-Type", "application/json")
            .header("Content-Length", content_lenth.to_string())
    }

    pub fn content(mut self, content_type: &'static str, content: impl Into<Cow<'static, [u8]>>) -> Self {
        let content: Cow<'static, [u8]> = content.into();
        let content_lenth = content.len();

        self.content = Some(content);
        self.header("Content-Type", content_type)
            .header("Content-Length", content_lenth.to_string())
    }
}


pub struct TestResponse(
    Response
);
impl TestResponse {
    fn new(response: Response) -> Self {
        Self(response)
    }
}
impl TestResponse {
    pub fn status(&self) -> Status {
        self.0.status
    }

    pub fn header(&self, name: &'static str) -> Option<&str> {
        self.0.headers.get(name)
    }
    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.headers.iter()
    }

    pub fn content(&self, content_type: &'static str) -> Option<&[u8]> {
        let _= self.0.headers.ContentType()?.starts_with(content_type)
            .then_some(())?;
        let bytes = self.0.content.as_bytes()?;
        assert_eq!(
            bytes.len(),
            self.0.headers.ContentLength()?.parse::<usize>().unwrap(),
            "Content-Length does not match the actual content length"
        );
        Some(bytes)
    }
    pub fn text(&self) -> Option<&str> {
        self.content("text/plain")
            .map(|bytes| std::str::from_utf8(bytes).expect(&f!(
                "Response content is not UTF-8: {}",
                bytes.escape_ascii()
            )))
    }
    pub fn html(&self) -> Option<&str> {
        self.content("text/html")
            .map(|bytes| std::str::from_utf8(bytes).expect(&f!(
                "Response content is not UTF-8: {}",
                bytes.escape_ascii()
            )))
    }
    pub fn json<'d, T: serde::Deserialize<'d>>(&'d self) -> Option<T> {
        self.content("application/json")
            .map(|bytes| serde_json::from_slice(bytes).expect(&f!(
                "Failed to deserialize json payload as {}: {}",
                std::any::type_name::<T>(),
                bytes.escape_ascii()
            )))
    }
}
