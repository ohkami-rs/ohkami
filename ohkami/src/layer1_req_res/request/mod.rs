mod path;    pub(crate) use path::Path;
mod queries; pub(crate) use queries::QueryParams;
mod store;   pub(crate) use store::Store;
mod parse_payload; pub use parse_payload::*;
mod from_request;  pub use from_request::*;
#[cfg(test)] mod _test_parse_payload;
#[cfg(test)] mod _test_parse;

use std::pin::Pin;
use byte_reader::Reader;
use crate::{
    __rt__::AsyncReader,
    layer0_lib::{Method, Slice, CowSlice, client_header, percent_decode_utf8}, websocket::UpgradeID
};


pub(crate) const METADATA_SIZE: usize = 1024;
pub(crate) const PAYLOAD_LIMIT: usize = 1 << 32;

pub struct Request {pub(crate) _metadata: [u8; METADATA_SIZE],
    pub method:            Method,
    pub headers:           client_header::Headers,
    pub(crate) path:       Path,
    queries:               QueryParams,
    payload:               Option<CowSlice>,
    store:                 Store,
    pub(crate) upgrade_id: Option<UpgradeID>,
}

impl Request {
    pub(crate) fn init() -> Self {
        Self {_metadata: [0; METADATA_SIZE],
            method:     Method::GET,
            path:       Path::init(),
            queries:    QueryParams::new(),
            headers:    client_header::Headers::init(),
            payload:    None,
            store:      Store::new(),
            upgrade_id: None,
        }
    }

    pub(crate) async fn read(
        mut self: Pin<&mut Self>,
        stream:   &mut (impl AsyncReader + Unpin),
    ) {
        stream.read(&mut self._metadata).await.unwrap();
        let mut r = Reader::new(&self._metadata);

        let method = Method::from_bytes(r.read_while(|b| b != &b' '));
        r.consume(" ").unwrap();
        
        let path = unsafe {// SAFETY: Just calling in request parsing
            Path::from_request_bytes(r.read_while(|b| b != &b'?' && b != &b' '))
        };

        let mut queries = QueryParams::new();
        if r.consume_oneof([" ", "?"]).unwrap() == 1 {
            while r.peek().is_some() {
                unsafe {// SAFETY: Just executing in request parsing
                    let key = Slice::from_bytes(r.read_while(|b| b != &b'='));
                    r.consume("=").unwrap();
                    let val = Slice::from_bytes(r.read_while(|b| b != &b'&' && b != &b' '));

                    queries.push_from_request_slice(key, val);

                    if r.consume_oneof(["&", " "]).unwrap() == 1 {break}
                }
            }
        }

        r.consume("HTTP/1.1\r\n").expect("Ohkami can only handle HTTP/1.1");

        let mut headers = client_header::Headers::init();
        while r.consume("\r\n").is_none() {
            if let Some(key) = client_header::Header::from_bytes(r.read_while(|b| b != &b':')) {
                r.consume(": ").unwrap();
                headers.insert(key, CowSlice::Ref(unsafe {
                    Slice::from_bytes(r.read_while(|b| b != &b'\r'))
                }));
            } else {
                r.consume(": ").unwrap();
                r.skip_while(|b| b != &b'\r');
            }
            r.consume("\r\n");
        }

        let content_length = headers.get(client_header::Header::ContentLength)
            .unwrap_or("")
            .as_bytes().into_iter()
            .fold(0, |len, b| 10*len + (*b - b'0') as usize);

        let payload = if content_length > 0 {
            Some(Request::read_payload(
                stream,
                &self._metadata,
                r.index,
                content_length.min(PAYLOAD_LIMIT),
            ).await)
        } else {None};

        self.method  = method;
        self.path    = path;
        self.queries = queries;
        self.headers = headers;
        self.payload = payload;
    }

    async fn read_payload(
        stream:       &mut (impl AsyncReader + Unpin),
        ref_metadata: &[u8],
        starts_at:    usize,
        size:         usize,
    ) -> CowSlice {#[cfg(debug_assertions)] assert!(starts_at < METADATA_SIZE, "ohkami can't handle requests if the total size of status and headers exceeds {METADATA_SIZE} bytes");
        if ref_metadata[starts_at] == 0 {// Some HTTP client requests as this
            let mut bytes = vec![0; size];
            stream.read(&mut bytes).await.unwrap();
            CowSlice::Own(bytes)
        } else if starts_at + size <= METADATA_SIZE {
            CowSlice::Ref(unsafe {Slice::new(ref_metadata.as_ptr().add(starts_at), size)})
        } else {
            (|| async move {
                let mut bytes = vec![0; size];
                let size_of_payload_in_metadata_bytes = METADATA_SIZE - starts_at;
                
                bytes[..size_of_payload_in_metadata_bytes].copy_from_slice(&ref_metadata[starts_at..]);
                stream.read(bytes[size_of_payload_in_metadata_bytes..].as_mut()).await.unwrap();
                
                CowSlice::Own(bytes)
            })().await
        }
    }
}

impl Request {
    #[inline] pub fn path(&self) -> std::borrow::Cow<'_, str> {
        percent_decode_utf8(unsafe {self.path.as_bytes()}).unwrap()
    }

    #[inline] pub fn query<'req, Value: FromParam<'req>>(&'req self, key: &str) -> Option<Result<Value, Value::Error>> {
        self.queries.get(key).map(Value::from_param)
    }
    pub fn append_query(&mut self, key: impl Into<std::borrow::Cow<'static, str>>, value: impl Into<std::borrow::Cow<'static, str>>) {
        self.queries.push(key, value)
    }

    #[inline] pub fn payload(&self) -> Option<&[u8]> {
        Some(unsafe {self.payload.as_ref()?.as_bytes()})
    }

    pub fn memorize<Value: Send + Sync + 'static>(&mut self, value: Value) {
        self.store.insert(value)
    }
    pub fn memorized<Value: Send + Sync + 'static>(&self) -> Option<&Value> {
        self.store.get()
    }
}

impl Request {
    #[inline(always)] pub(crate) unsafe fn path_bytes<'b>(&self) -> &'b [u8] {
        self.path.as_bytes()
    }
}

const _: () = {
    impl std::fmt::Debug for Request {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let queires = self.queries.iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>();

            let headers = self.headers.iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>();

            if let Some(payload) = self.payload() {
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

#[cfg(test)] const _: () = {
    impl PartialEq for Request {
        fn eq(&self, other: &Self) -> bool {
            self.method == other.method &&
            unsafe {self.path.as_bytes() == other.path.as_bytes()} &&
            self.queries == other.queries &&
            self.headers == other.headers &&
            self.payload == other.payload
        }
    }
};
