mod _test;

use std::borrow::Cow;
use std::collections::HashMap;
use std::{pin::Pin, future::Future, format as f};
use crate::{Response, Request, Method, layer0_lib::IntoCows};
use crate::{Ohkami, Context};

pub trait Testing {
    fn oneshot(&self, req: TestRequest) -> TestResponse;
}

impl Testing for Ohkami {
    fn oneshot(&self, request: TestRequest) -> TestResponse {
        let router = {
            let mut router = self.routes.clone();
            for (methods, fang) in &self.fangs {
                router = router.apply_fang(methods, fang.clone())
            }
            router.into_radix()
        };

        let res = async move {
            let mut req = Request::init();
            let mut req = unsafe {Pin::new_unchecked(&mut req)};
            req.as_mut().read(&mut &request.encode_request()[..]).await;
            router.handle(Context::new(), &mut req).await
        };

        TestResponse(Box::new(res))
    }
}

pub struct TestResponse<'test>(
    Box<dyn Future<Output = Response> + 'test>
); impl<'test> Future for TestResponse<'test> {
    type Output = Response;
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        unsafe {self.map_unchecked_mut(|tr| tr.0.as_mut())}
            .poll(cx)
    }
}

pub struct TestRequest {
    method:  Method,
    path:    Cow<'static, str>,
    queries: HashMap<Cow<'static, str>, Cow<'static, str>>,
    headers: HashMap<Cow<'static, str>, Cow<'static, str>>,
    content: Option<Cow<'static, str>>,
} impl TestRequest {
    fn encode_request(self) -> Vec<u8> {
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
            .fold(Vec::new(), |mut h, kv| if h.is_empty() {
                h.push(b'?'); h.extend_from_slice(kv.as_bytes()); h
            } else {
                h.push(b'&'); h.extend_from_slice(kv.as_bytes()); h
            });

        [
            method.as_bytes(), b" ", path.as_bytes(), &queries, b" HTTP/1.1\r\n",
            &headers,
            b"\r\n",
            content.unwrap_or(Cow::Borrowed("")).as_bytes()
        ].concat()
    }
}

macro_rules! new_test_request {
    ( $($method:ident)* ) => {$(
        #[allow(non_snake_case)]
        impl TestRequest {
            pub fn $method(path: impl IntoCows<'static>) -> Self {
                Self {
                    method:  Method::$method,
                    path:    path.into_cow(),
                    queries: HashMap::new(),
                    headers: HashMap::new(),
                    content: None,
                }
            }
        }
    )*};
} new_test_request! { GET PUT POST PATCH DELETE HEAD OPTIONS }

impl TestRequest {
    pub fn query(mut self, key: impl IntoCows<'static>, value: impl IntoCows<'static>) -> Self {
        self.queries.insert(key.into_cow(), value.into_cow());
        self
    }
    pub fn header(mut self, key: impl IntoCows<'static>, value: impl IntoCows<'static>) -> Self {
        self.headers.insert(key.into_cow(), value.into_cow());
        self
    }
}
impl TestRequest {
    pub fn json(mut self, json: impl serde::Serialize) -> Self {
        let content       = serde_json::to_string(&json).expect("Failed to serialize json");
        let content_lenth = content.len();

        self.content = Some(Cow::Owned(content));
        self.header("Content-Type", "application/json")
            .header("Content-Length", content_lenth.to_string())
    }
    pub fn json_lit(mut self, json: impl IntoCows<'static>) -> Self {
        let content = json.into_cow();
        let content_lenth = content.len();

        self.content = Some(content);
        self.header("Content-Type", "application/json")
            .header("Content-Length", content_lenth.to_string())
    }
}
