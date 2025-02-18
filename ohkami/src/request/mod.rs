mod method;
pub use method::Method;

mod path;
pub(crate) use path::Path;

mod query;
pub(crate) use query::QueryParams;

mod headers;
pub use headers::Headers as RequestHeaders;
#[allow(unused)]
pub use headers::Header as RequestHeader;

mod context;
use context::Context;

mod from_request; 
pub use from_request::*;

#[cfg(test)] mod _test_parse;
#[cfg(test)] mod _test_extract;
#[cfg(test)] mod _test_headers;

use ohkami_lib::{Slice, CowSlice};

#[cfg(feature="__rt_native__")]
use crate::__rt__::AsyncRead;

#[allow(unused)]
use {
    byte_reader::Reader,
    std::pin::Pin,
    std::borrow::Cow,
};


#[cfg(feature="__rt_native__")]
pub(crate) const BUF_SIZE: usize = 1 << 10;
#[cfg(feature="__rt_native__")]
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
/// - `ip`
/// 
/// and a `context`.
/// 
/// <br>
/// 
/// ## Usage
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
///     Ohkami::new((LogRequest,
///         "/".GET(|| async {"Hello, world!"})
///     )).howl("localhost:8000").await
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
    #[cfg(feature="__rt_native__")]
    pub(super/* for test */) __buf__: Box<[u8; BUF_SIZE]>,

    #[cfg(feature="rt_worker")]
    pub(super/* for test */) __url__: std::mem::MaybeUninit<::worker::Url>,

    #[cfg(feature="rt_lambda")]
    pub(super/* for test */) __query__: std::mem::MaybeUninit<Box<str>>,

    /// HTTP method of this request
    ///
    /// **Note** : In current version, custom HTTP methods are *not supported*,
    /// in other words, now Ohkami just knows `GET`, `PUT`, `POST`, `PATCH`,
    /// `DELETE`, `HEAD`, `OPTIONS`.
    pub method: Method,

    /// Request path of this request
    /// 
    /// - `.params()` to iterate path params
    /// - `.str()` to ( URL-decode and ) get as `&str`
    /// 
    /// **Note** : In current version, path with schema and origin in request line
    /// is *not supported*, in other words, now Ohkami just handles requests like
    /// `GET /path HTTP/1.1`, not `GET http://whatthe.fxxx/path HTTP/1.1`
    pub path: Path,

    /// Query params of this request
    /// 
    /// In handler, using a struct of expected schema
    /// with `ohkami::format::Query` is recommended for *type-safe*
    /// query parsing.
    /// 
    /// **Note** : Ohkami doesn't support multiple same query keys having each value
    /// like `?ids=1&ids=17&ids=42`.
    /// Please use, for instance, comma-separated format like
    /// `?ids=1,17,42` ( URL-encoded to `?ids=1%2C17%2C42` )
    pub query: QueryParams,

    /// Headers of this request
    /// 
    /// - `.{Name}()`, `.get("{Name}")` to get value
    /// - `.set().{Name}({action})`, `.set().x("{Name}", {action})` to mutate values
    /// 
    /// `{action}`:
    /// - just `{value}` to insert
    /// - `None` to remove
    /// - `header::append({value})` to append
    /// 
    /// `{value}`:
    /// - `String`
    /// - `&'static str`
    /// - `Cow<'static, str>`
    /// - `Some(Cow<'static, str>)`
    pub headers: RequestHeaders,

    pub payload: Option<CowSlice>,

    /// Request context.
    /// 
    /// `.set({T})` / `.get::<T>()` to memorize data to / retrieve it from request.
    /// 
    /// In `rt_worker`:
    /// - `.env()` to get `worker::Env`
    /// - `.worker()` to get `worker::Context`
    /// 
    /// In `rt_lambda`:
    /// - `.lambda()` to get `requestContext` of Lambda request
    pub context: Context,

    #[cfg(feature="__rt__")]
    /// Remote ( directly connected ) peer's IP address
    /// 
    /// Default value is `0.0.0.0`. this will be seen in testing or when Cloudlare Workers
    /// doesn't show ip.
    /// 
    /// **NOTE** : If a proxy is in front of Ohkami, this will be the proxy's address
    pub ip: std::net::IpAddr,
}

impl Request {
    #[cfg(feature="__rt__")]
    #[inline]
    pub(crate) fn init(
        #[cfg(feature="__rt_native__")]
        ip: std::net::IpAddr
    ) -> Self {
        Self {
            #[cfg(feature="__rt_native__")]
            ip,
            #[cfg(any(feature="rt_worker", feature="rt_lambda"))]
            ip: crate::util::IP_0000/* tetative */,

            #[cfg(feature="__rt_native__")]
            __buf__: Box::new([0; BUF_SIZE]),
            #[cfg(feature="rt_worker")]
            __url__: std::mem::MaybeUninit::uninit(),
            #[cfg(feature="rt_lambda")]
            __query__: std::mem::MaybeUninit::uninit(),

            method:  Method::GET,
            path:    Path::uninit(),
            query:   QueryParams::new(b""),
            headers: RequestHeaders::new(),
            payload: None,
            context: Context::init(),
        }
    }
    #[cfg(feature="__rt_native__")]
    #[inline]
    pub(crate) fn clear(&mut self) {
        if self.__buf__[0] != 0 {
            for b in &mut *self.__buf__ {
                match b {0 => break, _ => *b = 0}
            }
            self.path  = Path::uninit();
            self.query = QueryParams::new(b"");
            self.headers.clear();
            self.payload = None;
            self.context.clear();
        } /* else: just after `init`ed or `clear`ed */
    }

    #[cfg(feature="__rt_native__")]
    #[inline]
    pub(crate) async fn read(
        mut self: Pin<&mut Self>,
        stream:   &mut (impl AsyncRead + Unpin),
    ) -> Result<Option<()>, crate::Response> {
        use crate::Response;

        match stream.read(&mut *self.__buf__).await {
            Ok (0) => return Ok(None),
            Err(e) => return match e.kind() {
                std::io::ErrorKind::ConnectionReset => Ok(None),
                _ => Err((|err| {
                    crate::warning!("Failed to read stream: {err}");
                    Response::InternalServerError()
                })(e))
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
        
        self.path.init_with_request_bytes(r.read_while(|b| !matches!(b, b' ' | b'?')))?;

        if r.consume_oneof([" ", "?"]).unwrap() == 1 {
            self.query = QueryParams::new(r.read_while(|b| b != &b' '));
            r.advance_by(1);
        }

        r.consume("HTTP/1.1\r\n").ok_or_else(Response::HTTPVersionNotSupported)?;

        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").ok_or_else(Response::BadRequest)?;
            let value = CowSlice::Ref(Slice::from_bytes(r.read_while(|b| b != &b'\r')));
            r.consume("\r\n").ok_or_else(Response::BadRequest)?;

            if let Some(key) = RequestHeader::from_bytes(key_bytes) {
                self.headers.append(key, value);
            } else {
                self.headers.insert_custom(Slice::from_bytes(key_bytes), value)
            }
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

    #[cfg(feature="__rt_native__")]
    #[inline]
    async fn read_payload(
        stream:        &mut (impl AsyncRead + Unpin),
        remaining_buf: &[u8],
        size:          usize,
    ) -> CowSlice {
        let remaining_buf_len = remaining_buf.len();

        if remaining_buf_len == 0 || *unsafe {remaining_buf.get_unchecked(0)} == 0 {
            #[cfg(feature="DEBUG")] println!("\n[read_payload] case: remaining_buf.is_empty() || remaining_buf[0] == 0\n");

            let mut bytes = vec![0; size].into_boxed_slice();
            stream.read_exact(&mut bytes).await.unwrap();
            CowSlice::Own(bytes)

        } else if size <= remaining_buf_len {
            #[cfg(feature="DEBUG")] println!("\n[read_payload] case: starts_at + size <= BUF_SIZE\n");

            #[allow(unused_unsafe/* I don't know why but rustc sometimes put warnings to this unsafe as unnecessary */)]
            CowSlice::Ref(unsafe {
                Slice::new_unchecked(remaining_buf.as_ptr(), size)
            })

        } else {
            #[cfg(feature="DEBUG")] println!("\n[read_payload] case: else\n");

            let mut bytes = vec![0; size].into_boxed_slice();
            unsafe {// SAFETY: Here size > remaining_buf_len
                bytes.get_unchecked_mut(..remaining_buf_len).copy_from_slice(remaining_buf);
                stream.read_exact(bytes.get_unchecked_mut(remaining_buf_len..)).await.unwrap();
            }
            CowSlice::Own(bytes)
        }
    }


    #[cfg(debug_assertions/* for `ohkami::testing` */)]
    #[cfg(any(feature="rt_worker", feature="rt_lambda"))]
    /// Used in `testing` module
    pub(crate) async fn read(mut self: Pin<&mut Self>,
        raw_bytes: &mut &[u8]
    ) -> Result<Option<()>, crate::Response> {
        use crate::Response;

        self.ip = crate::util::IP_0000;

        let mut r = Reader::new(raw_bytes);

        match Method::from_bytes(r.read_while(|b| b != &b' ')) {
            None => return Ok(None),
            Some(method) => self.method = method
        }

        r.next_if(|b| *b==b' ').ok_or_else(Response::BadRequest)?;
        
        #[cfg(feature="rt_worker")] {
            self.__url__.write({
                let mut url = String::from("http://test.ohkami");
                url.push_str(std::str::from_utf8(r.read_while(|b| b != &b' ')).unwrap());
                ::worker::Url::parse(&url).unwrap()
            });
            // SAFETY: calling after `self.__url__` is already initialized
            unsafe {let __url__ = self.__url__.assume_init_ref();
                let path = Slice::from_bytes(__url__.path().as_bytes()).as_bytes();
                self.query = QueryParams::new(__url__.query().unwrap_or_default().as_bytes());
                self.path.init_with_request_bytes(path)?;
            }
        }

        #[cfg(feature="rt_lambda")] {
            let path_bytes = r.read_while(|b| b != &b' ' && b != &b'?');
            self.path.init_with_request_bytes(path_bytes)?;

            if r.next_if(|b| *b == b'?').is_some() {                
                self.__query__.write(
                    std::str::from_utf8(r.read_while(|b| b != &b' '))
                        .unwrap()
                        .to_owned()
                        .into_boxed_str()
                );
                // SAFETY: calling after `self.__query__` is already initialized
                unsafe {
                    self.query = QueryParams::new(self.__query__.assume_init_ref().as_bytes());
                }
            }

            r.next_if(|b| *b==b' ').ok_or_else(Response::BadRequest)?;
        }

        r.consume("HTTP/1.1\r\n").ok_or_else(Response::HTTPVersionNotSupported)?;

        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").ok_or_else(Response::BadRequest)?;
            let value = CowSlice::Own(r.read_while(|b| b != &b'\r').to_owned().into_boxed_slice());
            r.consume("\r\n").ok_or_else(Response::BadRequest)?;

            if let Some(key) = RequestHeader::from_bytes(key_bytes) {
                self.headers.append(key, value);
            } else {
                self.headers.insert_custom(
                    Slice::from_bytes(Box::leak(key_bytes.to_owned().into_boxed_slice())),
                    value
                )
            }
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
        self.context.load((ctx, env));

        self.method = Method::from_worker(req.method())
            .ok_or_else(|| Response::NotImplemented().with_text("ohkami doesn't support `CONNECT`, `TRACE` method"))?;

        self.__url__.write(req.url()
            .map_err(|_| Response::BadRequest().with_text("Invalid request URL"))?
        );
        #[cfg(feature="DEBUG")] worker::console_debug!("Load __url__: {:?}", self.__url__);

        // SAFETY: Just calling for request bytes and `self.__url__` is already initialized
        unsafe {let __url__ = self.__url__.assume_init_ref();
            let path = Slice::from_bytes(__url__.path().as_bytes()).as_bytes();
            self.query = QueryParams::new(__url__.query().unwrap_or_default().as_bytes());
            self.path.init_with_request_bytes(path)?;
        }

        self.headers.take_over(req.headers());

        self.payload = Some(CowSlice::Own(req.bytes().await
            .map_err(|_| Response::InternalServerError().with_text("Failed to read request payload"))?
            .into()
        ));

        if let Some(ip) = self.headers.get("cf-connecting-ip") {
            self.ip = ip.parse().unwrap(/* We think Cloudflare provides valid value here... */);
        }

        Ok(())
    }

    #[cfg(feature="rt_lambda")]
    pub(crate) fn take_over(mut self: Pin<&mut Self>,
        ::lambda_runtime::LambdaEvent {
            payload: req,
            context: _   
        }: ::lambda_runtime::LambdaEvent<crate::x_lambda::LambdaHTTPRequest>
    ) -> Result<(), lambda_runtime::Error> {
        self.__query__.write(req.rawQueryString.into_boxed_str()); unsafe {
            self.query = QueryParams::new(self.__query__.assume_init_ref().as_bytes());
        }

        self.context.load(req.requestContext); {
            let path_bytes = unsafe {
                let bytes = self.context.lambda().http.path.as_bytes();
                std::slice::from_raw_parts(bytes.as_ptr(), bytes.len())
            };
            self.path.init_with_request_bytes(path_bytes).map_err(|_| crate::util::ErrorMessage("unsupported path format".into()))?;
            self.method = self.context.lambda().http.method;
            self.ip = self.context.lambda().http.sourceIp;
        }

        self.headers = req.headers;
        if !req.cookies.is_empty() {
            self.headers.set().Cookie(req.cookies.join("; "));
        }

        if let Some(body) = req.body {
            self.payload = Some(CowSlice::Own(
                (if req.isBase64Encoded {
                    crate::util::base64_decode(body)?
                } else {
                    body.into_bytes()
                }).into_boxed_slice()
            ));
        }

        Result::<(), lambda_runtime::Error>::Ok(())
    }
}

impl Request {
    #[inline]
    pub fn payload(&self) -> Option<&[u8]> {
        self.payload.as_deref()
    }
}

const _: () = {
    impl std::fmt::Debug for Request {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut d = f.debug_struct("Request");
            let d = &mut d;

            #[cfg(feature="__rt__")] {
                d.field("ip", &self.ip);
            }

            d
                .field("method",  &self.method)
                .field("path",    &self.path.str())
                .field("queries", &self.query)
                .field("headers", &self.headers)
            ;

            if let Some(payload) = self.payload.as_ref().map(|cs| unsafe {cs.as_bytes()}) {
                d.field("payload", &String::from_utf8_lossy(payload));
            }
            
            d.finish()
        }
    }
};

#[cfg(feature="__rt__")]
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
