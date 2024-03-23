mod method;
pub use method::Method;

mod path;
pub(crate) use path::Path;

mod queries;
pub(crate) use queries::QueryParams;

mod headers;
pub use headers::Headers as RequestHeaders;

mod store;
pub(crate) use store::Store;
pub use store::Memory;

mod from_request; 
pub use from_request::*;

#[cfg(test)] mod _test_parse;

use ohkami_lib::{Slice, CowSlice, percent_decode_utf8};

use crate::typed::{Payload, PayloadType};

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
use {
    crate::__rt__::AsyncReader,
    std::pin::Pin,
    byte_reader::Reader,
};
#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub use {
    headers::Header as RequestHeader,
};

#[cfg(feature="websocket")]
use crate::websocket::UpgradeID;


pub(crate) const METADATA_SIZE: usize = 1024;

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub(crate) const PAYLOAD_LIMIT: usize = 1 << 32;

/// # HTTP Request
/// 
/// Composed of
/// 
/// - `method`
/// - `headers`
/// - `path`
/// - `queries`
/// - `payload`
/// 
/// and a `memory`.
/// 
/// <br>
/// 
/// ## Usages
/// 
/// ---
/// 
/// *in_fang.rs*
/// ```
/// use ohkami::{Request, Response, FrontFang};
/// 
/// struct LogRequest;
/// impl FrontFang for LogRequest {
///     type Error = std::convert::Infallible;
///     async fn bite(&self, req: &mut Request) -> Result<(), Self::Error> {
///         let method = req.method();
///         let path = req.path();
///         println!("{method} {path}");
///         Ok(())
///     }
/// }
/// ```
/// 
/// ---
/// 
/// *from_request.rs*
/// ```
/// use ohkami::{Request, FromRequest};
/// 
/// struct IsGET(bool);
/// 
/// impl<'req> FromRequest<'req> for IsGET {
///     type Error = std::convert::Infallible;
///     fn from_request(req: &'req Request) -> Result<Self, Self::Error> {
///         Ok(Self(
///             req.method().isGET()
///         ))
///     }
/// }
/// ```
pub struct Request {pub(crate) _metadata: [u8; METADATA_SIZE],
    method:          Method,
    /// Headers of this response
    /// 
    /// - `.{Name}()` to get the value
    /// - `.set().{Name}(〜)` to mutate the value
    ///   - `.set().{Name}({value})` to insert
    ///   - `.set().{Name}(None)` to remove
    ///   - `.set().{Name}(append({value}))` to append
    /// 
    /// `{value}`: `String`, `&'static str`, `Cow<&'static, str>`
    /// 
    /// ---
    /// 
    /// *`custom-header` feature required* :
    /// 
    /// - `.custom({Name})` to get the value
    /// - `.set().custom({Name}, 〜)` to mutate the value like standard headers
    pub headers:     RequestHeaders,
    pub(crate) path: Path,
    queries:         Option<Box<QueryParams>>,
    payload:         Option<CowSlice>,
    store:           Store,
}

impl Request {
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub(crate) fn init() -> Self {
        Self {_metadata: [0; METADATA_SIZE],
            method:     Method::GET,
            path:       unsafe {Path::null()},
            queries:    None,
            headers:    RequestHeaders::init(),
            payload:    None,
            store:      Store::new(),
            #[cfg(feature="websocket")] upgrade_id: None,
        }
    }

    #[cfg(any(feature="rt_tokio", feature="rt_async-std"))]
    pub(crate) async fn read(
        mut self: Pin<&mut Self>,
        stream:   &mut (impl AsyncReader + Unpin),
    ) -> Option<()> {
        if stream.read(&mut self._metadata).await.ok()? == 0 {return None};
        let mut r = Reader::new(&self._metadata);

        let method = Method::from_bytes(r.read_while(|b| b != &b' ')).unwrap();
        r.consume(" ").unwrap();
        
        let path = unsafe {// SAFETY: Just calling for request bytes
            Path::from_request_bytes(r.read_while(|b| b != &b'?' && b != &b' '))
        };

        let queries = (r.consume_oneof([" ", "?"]).unwrap() == 1)
            .then(|| Box::new({
                let q = QueryParams::new(r.read_while(|b| b != &b' '));
                #[cfg(debug_assertions)] {
                    r.consume(" ").unwrap();
                } #[cfg(not(debug_assertions))] {
                    r.advance_by(1)
                }
                q
            }));

        r.consume("HTTP/1.1\r\n").expect("Ohkami can only handle HTTP/1.1");

        let mut headers = RequestHeaders::init();
        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            if let Some(key) = RequestHeader::from_bytes(key_bytes) {
                r.consume(": ").unwrap();
                headers.insert(key, CowSlice::Ref(unsafe {
                    Slice::from_bytes(r.read_while(|b| b != &b'\r'))
                }));
            } else {
                #[cfg(not(feature="custom-header"))] {
                    r.consume(": ").unwrap();
                    r.skip_while(|b| b != &b'\r');
                }
                #[cfg(feature="custom-header")] {
                    let key = CowSlice::Ref(unsafe {Slice::from_bytes(key_bytes)});
                    r.consume(": ").unwrap();
                    headers.insert_custom(key, CowSlice::Ref(unsafe {
                        Slice::from_bytes(r.read_while(|b| b != &b'\r'))
                    }));
                }
            }
            r.consume("\r\n");
        }

        let content_length = headers.ContentLength()
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

        Some({
            self.method  = method;
            self.path    = path;
            self.queries = queries;
            self.headers = headers;
            self.payload = payload;
        })
    }

    #[cfg(any(feature="rt_tokio", feature="rt_async-std"))]
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
            CowSlice::Ref(unsafe {Slice::new_unchecked(ref_metadata.as_ptr().add(starts_at), size)})
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
    #[inline(always)] pub const fn method(&self) -> Method {
        self.method
    }

    /// Get request path as `Cow::Borrowed(&str)` if it's not percent-encoded, or, if encoded,
    /// decode it into `Cow::Owned(String)`.
    #[inline(always)] pub fn path(&self) -> std::borrow::Cow<'_, str> {
        percent_decode_utf8(unsafe {self.path.as_bytes()}).expect("Path is not UTF-8")
    }

    #[inline] pub fn queries<'req, Q: serde::Deserialize<'req>>(&'req self) -> Option<Result<Q, impl serde::de::Error>> {
        Some(unsafe {self.queries.as_ref()?.parse()})
    }
    #[inline] pub fn query<'req, Value: FromParam<'req>>(&'req self, key: &str) -> Option<Result<Value, Value::Error>> {
        let (_, value) = unsafe {self.queries.as_ref()?.iter()}.find(|(k, _)| k == key)?;
        Some(Value::from_param(value))
    }

    #[inline(always)] pub fn payload<
        'req, P: Payload + serde::Deserialize<'req> + 'req
    >(&'req self) -> Option<Result<P, impl serde::de::Error + 'req>> {
        self.headers.ContentType()?.starts_with(<<P as Payload>::Type as PayloadType>::MIME_TYPE)
            .then_some(<<P as Payload>::Type as PayloadType>::parse(unsafe {
                self.payload.as_ref()?.as_bytes()
            }))
    }

    /// Memorize any data within this request object
    #[inline(always)] pub fn memorize<Value: Send + Sync + 'static>(&mut self, value: Value) {
        self.store.insert(value)
    }
    /// Retrieve a data memorized in this request (using the type as key)
    #[inline(always)] pub fn memorized<Value: Send + Sync + 'static>(&self) -> Option<&Value> {
        self.store.get()
    }
}

impl Request {
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    #[inline(always)] pub(crate) unsafe fn internal_path_bytes<'b>(&self) -> &'b [u8] {
        self.path.as_internal_bytes()
    }
}

const _: () = {
    impl std::fmt::Debug for Request {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let queires = self.queries.as_ref().map(|q|
                unsafe {q.iter()}
                    .map(|(k, v)| format!("{k}: {v}"))
                    .collect::<Vec<_>>()
            ).unwrap_or_else(Vec::new);

            let headers = self.headers.iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>();

            if let Some(payload) = self.payload.as_ref().map(|cs| unsafe {cs.as_bytes()}) {
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

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
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
