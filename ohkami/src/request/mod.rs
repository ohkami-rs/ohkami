mod method;
pub use method::Method;

mod path;
pub(crate) use path::Path;

mod queries;
pub(crate) use queries::QueryParams;

mod headers;
pub use headers::Headers as RequestHeaders;
#[allow(unused)]
pub use headers::Header as RequestHeader;

mod memory;
pub(crate) use memory::Store;
pub use memory::Memory;

mod from_request; 
pub use from_request::*;

#[cfg(test)] mod _test_parse;
#[cfg(test)] mod _test_extract;
#[cfg(test)] mod _test_headers;

use ohkami_lib::{Slice, CowSlice};

use crate::typed::Payload;

#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
use {
    crate::__rt__::AsyncReader,
};
#[allow(unused)]
use {
    byte_reader::Reader,
    std::pin::Pin,
    std::borrow::Cow,
};

#[cfg(feature="websocket")]
use crate::websocket::UpgradeID;


#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub(crate) const BUF_SIZE: usize = 1024;
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
/// ```no_run
/// use ohkami::prelude::*;
/// 
/// #[derive(Clone)]
/// struct LogRequest;
/// impl FangAction for LogRequest {
///     async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
///         println!("{} {}", req.method, req.path);
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
///     fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
///         Some(Ok(Self(
///             req.method.isGET()
///         )))
///     }
/// }
/// ```
pub struct Request {
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub(super/* for test */) __buf__: Box<[u8; BUF_SIZE]>,

    #[cfg(feature="rt_worker")]
    pub(super/* for test */) __url__: std::mem::MaybeUninit<::worker::Url>,
    #[cfg(feature="rt_worker")]
    env: std::mem::MaybeUninit<::worker::Env>,
    #[cfg(feature="rt_worker")]
    ctx: std::mem::MaybeUninit<::worker::Context>,

    pub method: Method,
    pub path:   Path,
    pub query:  Option<QueryParams>,
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
    pub headers:        RequestHeaders,
    pub(crate) payload: Option<CowSlice>,
    store:              Store,
}

impl Request {
    #[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
    #[inline]
    pub(crate) fn init() -> Self {
        Self {
            #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
            __buf__: Box::new([0; BUF_SIZE]),

            #[cfg(feature="rt_worker")]
            __url__: std::mem::MaybeUninit::uninit(),
            #[cfg(feature="rt_worker")]
            env:     std::mem::MaybeUninit::uninit(),
            #[cfg(feature="rt_worker")]
            ctx:     std::mem::MaybeUninit::uninit(),

            method:  Method::GET,
            path:    Path::uninit(),
            query:   None,
            headers: RequestHeaders::init(),
            payload: None,
            store:   Store::init(),
        }
    }

    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub(crate) async fn read(
        mut self: Pin<&mut Self>,
        stream:   &mut (impl AsyncReader + Unpin),
    ) -> Result<Option<()>, crate::Response> {
        use crate::Response;

        match stream.read(&mut *self.__buf__).await {
            Ok (0) => return Ok(None),
            Err(e) => return match e.kind() {
                std::io::ErrorKind::ConnectionReset => Ok(None),
                _ => {eprintln!("Failed to"); Err((|| Response::InternalServerError())())}
            },
            _ => ()
        }

        let mut r = Reader::new(unsafe {
            // pass detouched bytes
            // to resolve immutable/mutable borrowing
            // 
            // SAFETY: `self.__buf__` itself is immutable
            Slice::from_bytes(&*self.__buf__).as_bytes()
        });

        match Method::from_bytes(r.read_while(|b| b != &b' ')) {
            None => return Ok(None),
            Some(method) => self.method = method
        }

        r.next_if(|b| *b==b' ').ok_or_else(Response::BadRequest)?;
        
        self.path = Path::from_request_bytes(r.read_while(|b| !matches!(b, b' ' | b'?')))?;

        if r.consume_oneof([" ", "?"]).unwrap() == 1 {
            self.query = Some(QueryParams::new(r.read_while(|b| b != &b' ')));
            r.advance_by(1);
        }

        r.consume("HTTP/1.1\r\n").ok_or_else(Response::HTTPVersionNotSupported)?;

        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").ok_or_else(Response::BadRequest)?;
            let value = Slice::from_bytes(r.read_while(|b| b != &b'\r'));
            if let Some(key) = RequestHeader::from_bytes(key_bytes) {
                self.headers.append(key, CowSlice::Ref(value));
            } else {
                match key_bytes {
                    b"Cookie" | b"cookie" => self.headers.append_cookie(value),
                    _ => self.headers.append_custom(
                        Slice::from_bytes(key_bytes),
                        CowSlice::Ref(value)
                    )
                }
            }
            r.consume("\r\n").ok_or_else(Response::BadRequest)?;
        }

        let content_length = match self.headers.get_raw(RequestHeader::ContentLength) {
            Some(v) => unsafe {v.as_bytes()}.into_iter().fold(0, |len, b| 10*len + (*b - b'0') as usize),
            None    => 0,
        };
        match content_length {
            0 => (),
            PAYLOAD_LIMIT.. => return Err((|| Response::PayloadTooLarge())()),
            _ => self.payload = Some(Request::read_payload(
                stream,
                r.remaining(),
                content_length,
            ).await)
        }

        Ok(Some(()))
    }

    #[cfg(any(feature="rt_tokio", feature="rt_async-std"))]
    async fn read_payload(
        stream:        &mut (impl AsyncReader + Unpin),
        remaining_buf: &[u8],
        size:          usize,
    ) -> CowSlice {
        if remaining_buf.is_empty() || remaining_buf[0] == 0 {
            #[cfg(feature="DEBUG")] println!("\n[read_payload] case: remaining_buf.is_empty() || remaining_buf[0] == 0\n");

            let mut bytes = vec![0; size].into_boxed_slice();
            stream.read_exact(&mut bytes).await.unwrap();
            CowSlice::Own(bytes)

        } else if size <= remaining_buf.len() {
            #[cfg(feature="DEBUG")] println!("\n[read_payload] case: starts_at + size <= BUF_SIZE\n");

            #[allow(unused_unsafe/* I don't know why but rustc sometimes put warnings to this unsafe as unnecessary */)]
            CowSlice::Ref(unsafe {
                Slice::new_unchecked(remaining_buf.as_ptr(), size)
            })

        } else {
            #[cfg(feature="DEBUG")] println!("\n[read_payload] case: else\n");

            let mut bytes = vec![0; size].into_boxed_slice();
            let remaining_buf_len = remaining_buf.len();
            unsafe {// SAFETY: Here size > remaining_buf_len
                bytes.get_unchecked_mut(..remaining_buf_len).copy_from_slice(remaining_buf);
                stream.read_exact(bytes.get_unchecked_mut(remaining_buf_len..)).await.unwrap();
            }
            CowSlice::Own(bytes)
        }
    }

    #[cfg(feature="rt_worker")]
    #[cfg(feature="testing")]
    pub(crate) async fn read(mut self: Pin<&mut Self>,
        raw_bytes: &mut &[u8]
    ) -> Result<Option<()>, crate::Response> {
        use crate::Response;

        let mut r = Reader::new(raw_bytes);

        match Method::from_bytes(r.read_while(|b| b != &b' ')) {
            None => return Ok(None),
            Some(method) => self.method = method
        }

        r.next_if(|b| *b==b' ').ok_or_else(Response::BadRequest)?;
        
        self.__url__.write({
            let mut url = String::from("http://test.ohkami");
            url.push_str(std::str::from_utf8(r.read_while(|b| b != &b' ')).unwrap());
            ::worker::Url::parse(&url).unwrap()
        });
        // SAFETY: Just calling for request bytes and `self.__url__` is already initialized
        unsafe {let __url__ = self.__url__.assume_init_ref();
            let path  = Path::from_request_bytes(__url__.path().as_bytes()).unwrap();
            let query = __url__.query().map(|str| QueryParams::new(str.as_bytes()));
            self.path  = path;
            self.query = query;
        }

        r.consume("HTTP/1.1\r\n").ok_or_else(Response::HTTPVersionNotSupported)?;

        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").ok_or_else(Response::BadRequest)?;
            let value = Slice::from_bytes(r.read_while(|b| b != &b'\r'));
            if let Some(key) = RequestHeader::from_bytes(key_bytes) {
                self.headers.append(key, CowSlice::Ref(value));
            } else {
                match key_bytes {
                    b"Cookie" | b"cookie" => self.headers.append_cookie(value),
                    _ => self.headers.append_custom(
                        Slice::from_bytes(key_bytes),
                        CowSlice::Ref(value)
                    )
                }
            }
            r.consume("\r\n").ok_or_else(Response::BadRequest)?;
        }

        let content_length = match self.headers.get_raw(RequestHeader::ContentLength) {
            Some(v) => unsafe {v.as_bytes()}.into_iter().fold(0, |len, b| 10*len + (*b - b'0') as usize),
            None    => 0,
        };
        self.payload = (content_length > 0).then(||
            CowSlice::Own(r.remaining().into())
        );

        Ok(Some(()))
    }

    #[cfg(feature="rt_worker")]
    pub(crate) async fn take_over(mut self: Pin<&mut Self>,
        mut req: ::worker::Request,
        env:     ::worker::Env,
        ctx:     ::worker::Context,
    ) -> Result<(), crate::Response> {use crate::Response;
        self.env.write(env);
        self.ctx.write(ctx);

        self.method = Method::from_worker(req.method())
            .ok_or_else(|| Response::NotImplemented().with_text("ohkami doesn't support `CONNECT`, `TRACE` method"))?;

        self.__url__.write(req.url()
            .map_err(|_| Response::BadRequest().with_text("Invalid request URL"))?
        );
        #[cfg(feature="DEBUG")] worker::console_debug!("Load __url__: {:?}", self.__url__);

        // SAFETY: Just calling for request bytes and `self.__url__` is already initialized
        unsafe {let __url__ = self.__url__.assume_init_ref();
            let path  = Path::from_request_bytes(__url__.path().as_bytes()).unwrap();
            let query = __url__.query().map(|str| QueryParams::new(str.as_bytes()));
            self.path  = path;
            self.query = query;
        }

        self.headers.take_over(req.headers());

        self.payload = Some(CowSlice::Own(req.bytes().await
            .map_err(|_| Response::InternalServerError().with_text("Failed to read request payload"))?
            .into()
        ));

        Ok(())
    }
}

#[cfg(feature="rt_worker")]
impl Request {
    #[inline]
    pub fn env(&self) -> &::worker::Env {
        unsafe {self.env.assume_init_ref()}
    }
    #[inline]
    pub fn context(&self) -> &::worker::Context {
        unsafe {self.ctx.assume_init_ref()}
    }
}

impl Request {
    #[inline(always)] pub fn payload<
        'req, P: Payload + serde::Deserialize<'req> + 'req
    >(&'req self) -> Option<Result<P, impl serde::de::Error + 'req>> {
        P::extract(self)
    }

    /// Memorize any data within this request object
    #[inline] pub fn memorize<Value: Send + Sync + 'static>(&mut self, value: Value) {
        self.store.insert(value)
    }
    /// Retrieve a data memorized in this request (using the type as key)
    #[inline] pub fn memorized<Value: Send + Sync + 'static>(&self) -> Option<&Value> {
        self.store.get()
    }
}

const _: () = {
    impl std::fmt::Debug for Request {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if let Some(payload) = self.payload.as_ref().map(|cs| unsafe {cs.as_bytes()}) {
                f.debug_struct("Request")
                    .field("method",  &self.method)
                    .field("path",    &self.path.str())
                    .field("queries", &self.query)
                    .field("headers", &self.headers)
                    .field("payload", &String::from_utf8_lossy(payload))
                    .finish()
            } else {
                f.debug_struct("Request")
                    .field("method",  &self.method)
                    .field("path",    &self.path.str())
                    .field("queries", &self.query)
                    .field("headers", &self.headers)
                    .finish()
            }
        }
    }
};

#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
#[cfg(test)] const _: () = {
    impl PartialEq for Request {
        fn eq(&self, other: &Self) -> bool {
                self.method == other.method &&
                unsafe {self.path.normalized_bytes() == other.path.normalized_bytes()} &&
                self.query == other.query &&
                self.headers == other.headers &&
                self.payload == other.payload
        }
    }
};
