#[cfg(test)] mod _test;

use crate::{Response, Request, Ohkami};
use crate::layer0_lib::{Method, Status};
use crate::layer1_req_res::ResponseHeader;

use std::borrow::Cow;
use std::collections::HashMap;
use std::{pin::Pin, future::Future, format as f};


pub trait Testing {
    fn oneshot(&self, req: TestRequest) -> Oneshot;

    // #[cfg(feature="websocket")]
    // fn oneshot_and_on_upgrade(&self, req: TestRequest) -> OneshotAndUpgraded;
}

pub struct Oneshot(
    Box<dyn Future<Output = TestResponse>>
); impl Future for Oneshot {
    type Output = TestResponse;
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        unsafe {self.map_unchecked_mut(|this| this.0.as_mut())}.poll(cx)
    }
}

impl Testing for Ohkami {
    fn oneshot(&self, request: TestRequest) -> Oneshot {
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

            #[cfg(not(feature="websocket"))]
            let res = router.handle(&mut req).await;
            #[cfg(feature="websocket")]
            let (res, _) = router.handle(&mut req).await;

            TestResponse::new(res)
        };

        Oneshot(Box::new(res))
    }

//    #[cfg(feature="websocket")]
//    fn oneshot_and_on_upgrade(
//        &self,
//        request: TestRequest,
//
//    ) -> OneshotAndUpgraded {
//        use crate::websocket::{reserve_upgrade_in_test, assume_upgradable_in_test, WebSocketContext};
//
//        let router = {
//            let mut router = self.routes.clone();
//            for (methods, fang) in &self.fangs {
//                router = router.apply_fang(methods, fang.clone())
//            }
//            router.into_radix()
//        };
//
//        let res_and_socket = async move {
//            let mut req = Request::init();
//            let mut req = unsafe {Pin::new_unchecked(&mut req)};
//            req.as_mut().read(&mut &request.encode_request()[..]).await;
//
//            let (res, upgrade_id) = router.handle(Context::new(), &mut req).await;
//            match upgrade_id {
//                None     => (TestResponse::new(res), None),
//                Some(id) => {
//                    let (client, server) = TestStream::new_pair();
//                    unsafe {reserve_upgrade_in_test(id, server)};
//
//                    let server = assume_upgradable_in_test(id).await;
//                    let ctx = WebSocketContext::new(Context {
//                        upgrade_id,
//                        ..Context::new()
//                    }, &mut req);
//                    
//                    (TestResponse::new(res), Some(TestWebSocket::new(client)))
//                },
//            }
//        };
//
//        OneshotAndUpgraded(Box::new(res_and_socket))
//    }
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
            .fold(Vec::new(), |mut h, kv| {
                h.extend_from_slice(kv.as_bytes()); h
            });

        [
            method.as_str().as_bytes(), b" ", path.as_bytes(), &queries, b" HTTP/1.1\r\n",
            &headers,
            b"\r\n",
            content.unwrap_or(Cow::Borrowed("")).as_bytes()
        ].concat()
    }
} macro_rules! new_test_request {
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
} impl TestRequest {
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
        let content       = serde_json::to_string(&json).expect("Failed to serialize json");
        let content_lenth = content.len();

        self.content = Some(Cow::Owned(content));
        self.header("Content-Type", "application/json")
            .header("Content-Length", content_lenth.to_string())
    }
    pub fn json_lit(mut self, json: impl Into<Cow<'static, str>>) -> Self {
        let content = json.into();
        let content_lenth = content.len();

        self.content = Some(content);
        self.header("Content-Type", "application/json")
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
        let name_bytes = name.split('-').map(|section| {
            if section.eq_ignore_ascii_case("ETag") {
                f!("ETag")
            } else if section.eq_ignore_ascii_case("WebSocket") {
                f!("WebSocket")
            } else {
                let mut section_chars = section.chars();
                let first = section_chars.next().expect("Found `--` in header name").to_ascii_uppercase();
                section_chars.fold(
                    String::from(first),
                    |mut section, ch| {section.push(ch); section}
                )
            }
        }).collect::<String>();
        self.0.headers.get(ResponseHeader::from_bytes(name_bytes.as_bytes())?)
    }

    pub fn text(&self) -> Option<&str> {
        if self.0.headers.ContentType()?.starts_with("text/plain") {
            let body = self.0.content.as_ref()?;
            Some(std::str::from_utf8(body).expect(&f!("Response content is not UTF-8: {}", body.escape_ascii())))
        } else {None}
    }
    pub fn html(&self) -> Option<&str> {
        if self.0.headers.ContentType()?.starts_with("text/html") {
            let body = self.0.content.as_ref()?;
            Some(std::str::from_utf8(body).expect(&f!("Response content is not UTF-8: {}", body.escape_ascii())))
        } else {None}
    }
    pub fn json<'d, JSON: serde::Deserialize<'d>>(&'d self) -> Option<serde_json::Result<JSON>> {
        if self.0.headers.ContentType()?.starts_with("application/json") {
            let body = self.0.content.as_ref()?;
            Some(serde_json::from_slice(body))
        } else {None}
    }
}
