mod method;
pub use method::Method;

mod path;
pub(crate) use path::Path;

mod queries;
pub(crate) use queries::QueryParams;

mod headers;
pub use headers::Headers as RequestHeaders;

mod memory;
pub(crate) use memory::Store;
pub use memory::Memory;

mod from_request; 
pub use from_request::*;

#[cfg(test)] mod _test_parse;

use ohkami_lib::{Slice, CowSlice, percent_decode_utf8};

use crate::typed::{Payload, PayloadType};

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
use {
    crate::__rt__::AsyncReader,
    byte_reader::Reader,
};
#[cfg(feature="rt_worker")]
use {
    std::sync::OnceLock,
};
#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
use {
    std::pin::Pin,
};
#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
pub use {
    headers::Header as RequestHeader,
};

#[cfg(feature="websocket")]
use crate::websocket::UpgradeID;


#[cfg(feature="rt_worker")]
static WORKER_ENV: OnceLock<::worker::Env> = OnceLock::new();
#[cfg(feature="rt_worker")]
static WORKER_CTX: OnceLock<::worker::Context> = OnceLock::new();


pub(crate) const BUF_SIZE: usize = 1024;

#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
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
/// ```no_run
/// use ohkami::prelude::*;
/// 
/// #[derive(Clone)]
/// struct LogRequest;
/// impl FangAction for LogRequest {
///     async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
///         println!("{} {}", req.method(), req.path());
///         Ok(())
///     }
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::with(LogRequest,
///         "/".GET(|| async {"Hello, world!"})
///     ).howl("localhost:8000").await
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
pub struct Request {
    method: Method,

    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub(crate) __buf__: Box<[u8; BUF_SIZE]>,
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    path:    Path,
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    queries: Option<Box<QueryParams>>,

    #[cfg(feature="rt_worker")]
    url: ::worker::Url,

    /// Headers of this request
    /// 
    /// - `.{Name}()`, `.custom({Name})` to get the value
    /// - `.set().{Name}({action})`, `.set().custom({Name}, {action})` to mutate the values
    /// 
    /// ---
    /// 
    /// `{action}`:
    /// - just `{value}` to insert
    /// - `None` to remove
    /// - `append({value})` to append
    /// 
    /// `{value}`: `String`, `&'static str`, `Cow<&'static, str>`
    pub headers:     RequestHeaders,
    payload:         Option<CowSlice>,
    store:           Store,
}

impl Request {
    #[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
    pub(crate) fn init() -> Self {
        Self {
            method:  Method::GET,

            #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
            __buf__: Box::new([0; BUF_SIZE]),
            #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
            path:    unsafe {Path::null()},
            #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
            queries: None,

            #[cfg(feature="rt_worker")]
            url:     ::worker::Url::parse("http://nul.l").unwrap(),

            headers: RequestHeaders::init(),
            payload: None,
            store:   Store::new(),
        }
    }

    #[cfg(any(feature="rt_tokio", feature="rt_async-std"))]
    pub(crate) async fn read(
        mut self: Pin<&mut Self>,
        stream:   &mut (impl AsyncReader + Unpin),
    ) -> Option<()> {
        if stream.read(&mut *self.__buf__).await.ok()? == 0 {return None};
        let mut r = Reader::new(&*self.__buf__);

        let method = Method::from_bytes(r.read_while(|b| b != &b' '))?;
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
            r.consume(": ").unwrap();
            if let Some(key) = RequestHeader::from_bytes(key_bytes) {
                headers.insert(key, CowSlice::Ref(unsafe {
                    Slice::from_bytes(r.read_while(|b| b != &b'\r'))
                }));
            } else {
                headers.insert_custom(
                    CowSlice::Ref(unsafe {Slice::from_bytes(key_bytes)}),
                    CowSlice::Ref(unsafe {Slice::from_bytes(r.read_while(|b| b != &b'\r'))})
                );
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
                &*self.__buf__,
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
    ) -> CowSlice {
        #[cfg(debug_assertions)] {
            assert!(starts_at < BUF_SIZE, "ohkami can't handle requests if the total size of status and headers exceeds {BUF_SIZE} bytes");
        }

        if ref_metadata[starts_at] == 0 {
            #[cfg(feature="DEBUG")] println!("\n[read_payload] case: ref_metadata[starts_at] == 0\n");

            let mut bytes = vec![0; size];
            stream.read_exact(&mut bytes).await.unwrap();
            CowSlice::Own(bytes)

        } else if starts_at + size <= BUF_SIZE {
            #[cfg(feature="DEBUG")] println!("\n[read_payload] case: starts_at + size <= BUF_SIZE\n");

            CowSlice::Ref(unsafe {Slice::new_unchecked(ref_metadata.as_ptr().add(starts_at), size)})

        } else {
            #[cfg(feature="DEBUG")] println!("\n[read_payload] case: else\n");

            let mut bytes = vec![0; size];
            let size_of_payload_in_metadata_bytes = BUF_SIZE - starts_at;
                
            bytes[..size_of_payload_in_metadata_bytes].copy_from_slice(&ref_metadata[starts_at..]);
            stream.read_exact(bytes[size_of_payload_in_metadata_bytes..].as_mut()).await.unwrap();
                
            CowSlice::Own(bytes)
        }
    }

    #[cfg(feature="rt_worker")]
    pub async fn take_over(mut self: Pin<&mut Self>,
        mut req: ::worker::Request,
        env:     ::worker::Env,
        ctx:     ::worker::Context,
    ) -> Result<(), crate::Response> {
        use crate::Response;

        self.headers.take_over(req.headers());
        self.method  = Method::from_worker(req.method())
            .ok_or_else(|| Response::NotImplemented().text("ohkami doesn't support `CONNECT`, `TRACE` method"))?;
        self.url = req.url()
            .map_err(|_| Response::InternalServerError().text("Invalid request URL"))?;
        self.payload = Some(CowSlice::Own(req.bytes().await
            .map_err(|_| Response::InternalServerError().text("Failed to read request payload"))?));

        WORKER_ENV.set(env).ok().unwrap();
        WORKER_CTX.set(ctx).ok().unwrap();

        Ok(())
    }
}

impl Request {
    #[inline(always)] pub const fn method(&self) -> Method {
        self.method
    }

    /// Get request path as `Cow::Borrowed(&str)` if it's not percent-encoded, or, if encoded,
    /// decode it into `Cow::Owned(String)`.
    #[inline(always)] pub fn path(&self) -> std::borrow::Cow<'_, str> {
        #[cfg(any(feature="rt_tokio",feature="rt_async-std"))] {
            percent_decode_utf8(unsafe {self.path.as_bytes()}).expect("Path is not UTF-8")
        }
        #[cfg(feature="rt_worker")] {
            percent_decode_utf8(self.url.path().as_bytes()).expect("Path is not UTF-8")            
        }
    }

    #[inline] pub fn queries<'req, Q: serde::Deserialize<'req>>(&'req self) -> Option<Result<Q, impl serde::de::Error>> {
        #[cfg(any(feature="rt_tokio",feature="rt_async-std"))] {
            Some(unsafe {self.queries.as_ref()?.parse()})
        }
        #[cfg(feature="rt_worker")] {
            self.url.query().map(|str| ohkami_lib::serde_urlencoded::from_bytes(str.as_bytes()))
        }
    }
    #[inline] pub fn query<'req, Value: FromParam<'req>>(&'req self, key: &str) -> Option<Result<Value, Value::Error>> {
        #[cfg(any(feature="rt_tokio",feature="rt_async-std"))] {
            let (_, value) = unsafe {self.queries.as_ref()?.iter()}.find(|(k, _)| k == key)?;
            Some(Value::from_param(value))
        }
        #[cfg(feature="rt_worker")] {
            let (_, value) = self.url.query_pairs().find(|(k, _)| k == key)?;
            Some(Value::from_param(value))
        }
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
    #[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
    #[inline(always)] pub(crate) unsafe fn internal_path_bytes(&self) -> &[u8] {
        #[cfg(any(feature="rt_tokio",feature="rt_async-std"))] {
            self.path.as_internal_bytes()
        }
        #[cfg(feature="rt_worker")] {
            self.url.path().as_bytes()
        }
    }
}

const _: () = {
    impl std::fmt::Debug for Request {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let queries = {
                #[cfg(any(feature="rt_tokio",feature="rt_async-std"))] {
                    self.queries.as_ref().map(|q|
                        unsafe {q.iter()}
                            .map(|(k, v)| format!("{k}: {v}"))
                            .collect::<Vec<_>>()
                    )
                }
                #[cfg(feature="rt_worker")] {
                    self.url.query().is_some().then(|| self.url.query_pairs()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                    )
                }
            }.unwrap_or_else(Vec::new);

            let headers = self.headers.iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>();

            if let Some(payload) = self.payload.as_ref().map(|cs| unsafe {cs.as_bytes()}) {
                f.debug_struct("Request")
                    .field("method",  &self.method)
                    .field("path",    &self.path())
                    .field("queries", &queries)
                    .field("headers", &headers)
                    .field("payload", &String::from_utf8_lossy(payload))
                    .finish()
            } else {
                f.debug_struct("Request")
                    .field("method",  &self.method)
                    .field("path",    &self.path())
                    .field("queries", &queries)
                    .field("headers", &headers)
                    .finish()
            }
        }
    }
};

#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
#[cfg(test)] const _: () = {
    impl PartialEq for Request {
        fn eq(&self, other: &Self) -> bool {
            #[cfg(any(feature="rt_tokio",feature="rt_async-std"))] {
                self.method == other.method &&
                unsafe {self.path.as_bytes() == other.path.as_bytes()} &&
                self.queries == other.queries &&
                self.headers == other.headers &&
                self.payload == other.payload
            }
            #[cfg(feature="rt_worker")] {
                self.method  == other.method  &&
                self.url     == other.url     &&
                self.headers == other.headers &&
                self.payload == other.payload
            }
        }
    }
};
