mod method;

pub use method::Method;

mod path;
pub(crate) use path::Path;

mod query;
pub(crate) use query::QueryParams;

mod headers;
#[allow(unused)]
pub use headers::Header as RequestHeader;
pub use headers::Headers as RequestHeaders;

mod context;
use context::Context;

mod from_request;
pub use from_request::FromRequest;

#[cfg(test)]
mod _test_extract;
#[cfg(test)]
mod _test_headers;
#[cfg(test)]
mod _test_parse;

use ohkami_lib::CowSlice;
#[cfg(feature = "__rt__")]
use ohkami_lib::Slice;

#[allow(unused)]
use {crate::Response, byte_reader::Reader, std::borrow::Cow, std::pin::Pin};

#[cfg(feature = "__rt_native__")]
use crate::__rt__::AsyncRead as ReadableStream;
#[cfg(all(feature = "__rt_edge__", debug_assertions))]
use testing::ReadableStream; /* **only for testing** for edge runtimes */
#[cfg(all(feature = "__rt_edge__", debug_assertions))]
mod testing {
    pub(crate) trait ReadableStream {
        async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
        async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()>;
    }
    impl<T: std::io::Read + ?Sized> ReadableStream for T {
        async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            std::io::Read::read(self, buf)
        }
        async fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
            std::io::Read::read_exact(self, buf)
        }
    }
}

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
///     )).run("localhost:8000").await
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
    pub(super) __buf__: Box<[u8]>,

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
    /// - `.{name}()`, `.get("{name}")` to get value
    /// - `.set().{name}({action})`, `.set().x("{name}", {action})` to mutate values
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

    #[cfg(feature = "__rt__")]
    /// Remote ( directly connected ) peer's IP address
    ///
    /// Default value is `0.0.0.0`. this will be seen in testing or when Cloudlare Workers
    /// doesn't show ip.
    ///
    /// **NOTE** : If a proxy is in front of Ohkami, this will be the proxy's address
    pub ip: std::net::IpAddr,
}

impl Request {
    #[cfg(feature = "__rt__")]
    #[inline]
    fn get_payload_size(
        &self,
        #[cfg(feature = "__rt_native__")] config: &crate::Config,
    ) -> Result<Option<std::num::NonZeroUsize>, crate::Response> {
        let Some(size) = self
            .headers
            .content_length()
            .map(|s| s.parse().map_err(|_| Response::BadRequest()))
            .transpose()?
            .and_then(std::num::NonZeroUsize::new)
        else {
            return Ok(None);
        };

        // reject GET/HEAD/OPTIONS requests having positive `Content-Length`
        // as `400 Bad Request` for security reasons
        if matches!(self.method, Method::GET | Method::HEAD | Method::OPTIONS) {
            return Err(Response::BadRequest());
        }

        #[cfg(feature = "__rt_native__")]
        // reject requests having `Content-Length` larger than this limit
        // as `413 Payload Too Large` for security reasons
        if size.get() > config.request_payload_limit {
            return Err(Response::PayloadTooLarge());
        }

        Ok(Some(size))
    }

    #[cfg(feature = "__rt__")]
    #[inline]
    pub(crate) fn uninit(
        #[cfg(feature = "__rt_native__")] ip: std::net::IpAddr,
        #[cfg(feature = "__rt_native__")] config: &crate::Config,
    ) -> Self {
        let ip = {
            #[cfg(feature = "__rt_native__")]
            {
                ip
            }
            #[cfg(all(feature = "__rt_edge__", debug_assertions))]
            {
                // tenatively use (will be replaced with real ip later (e.g., in `take_over`))
                crate::util::IP_0000
            }
        };

        let buf_size = {
            #[cfg(feature = "__rt_native__")]
            {
                config.request_bufsize
            }
            #[cfg(all(feature = "__rt_edge__", debug_assertions))]
            {
                // 4 KiB **only for testing** implementation for edge runtimes
                1 << 12
            }
            #[cfg(all(feature = "__rt_edge__", not(debug_assertions)))]
            {
                // `__buf__` is never actually used in edge runtimes
                0
            }
        };

        Self {
            ip,
            __buf__: vec![0u8; buf_size].into_boxed_slice(),
            method: Method::GET,
            path: Path::uninit(),
            query: QueryParams::new(b""),
            headers: RequestHeaders::new(),
            payload: None,
            context: Context::init(),
        }
    }

    #[cfg(feature = "__rt_native__")]
    #[inline(always)]
    pub(crate) fn clear(&mut self) {
        if self.__buf__[0] != 0 {
            for b in &mut *self.__buf__ {
                match b {
                    0 => break,
                    _ => *b = 0,
                }
            }
            self.path = Path::uninit();
            self.query = QueryParams::new(b"");
            self.headers.clear();
            self.payload = None;
            self.context.clear();
        } /* else: just after `init`ed or `clear`ed */
    }

    #[cfg(any(
        feature = "__rt_native__",
        all(feature = "__rt_edge__", debug_assertions)
    ))]
    pub(crate) async fn read(
        mut self: Pin<&mut Self>,
        stream: &mut (impl ReadableStream + Unpin),
        #[cfg_attr(all(feature = "__rt_edge__", debug_assertions), allow(unused))]
        config: &crate::Config,
    ) -> Result<Option<()>, Response> {
        match stream.read(&mut self.__buf__).await {
            Ok(0) => return Ok(None),
            Err(e) => {
                return match e.kind() {
                    std::io::ErrorKind::ConnectionReset => Ok(None),
                    _ => Err({
                        crate::WARNING!("Failed to read stream: {e}");
                        Response::InternalServerError()
                    }),
                };
            }
            _ => (),
        }

        let mut r = Reader::new(unsafe {
            // pass detouched bytes
            // to avoid compile-errors around immutable/mutable borrowing
            //
            // SAFETY: `self.__buf__` itself is immutable after this point
            Slice::from_bytes(&self.__buf__).as_bytes()
        });

        match Method::from_bytes(r.read_while(|b| b != &b' ')) {
            None => return Ok(None),
            Some(method) => self.method = method,
        }

        r.next_if(|b| *b == b' ').ok_or_else(Response::BadRequest)?;

        self.path
            .init_with_request_bytes(r.read_while(|b| !matches!(b, b' ' | b'?')))?;

        if r.consume_oneof([" ", "?"]).unwrap() == 1 {
            self.query = QueryParams::new(r.read_while(|b| b != &b' '));
            r.advance_by(1);
        }

        r.consume("HTTP/1.1\r\n")
            .ok_or_else(Response::HTTPVersionNotSupported)?;

        while r.consume("\r\n").is_none() {
            let key_bytes = r.read_while(|b| b != &b':');
            r.consume(": ").ok_or_else(|| {
                #[cfg(feature = "__rt_native__")]
                crate::WARNING!(
                    "\
                    [Request::read] Unexpected end of headers! \
                    Maybe request buffer size is not enough. \
                    Try setting `request_bufsize` of Config, \
                    or `OHKAMI_REQUEST_BUFSIZE` environment variable, \
                    to a larger value (default: {}).\
                ",
                    crate::Config::default().request_bufsize
                );
                Response::RequestHeaderFieldsTooLarge()
            })?;

            let value = CowSlice::Ref(Slice::from_bytes(r.read_while(|b| b != &b'\r')));
            r.consume("\r\n").ok_or_else(|| {
                #[cfg(feature = "__rt_native__")]
                crate::WARNING!(
                    "\
                    [Request::read] Unexpected end of headers! \
                    Maybe request buffer size is not enough. \
                    Try setting `request_bufsize` of Config, \
                    or `OHKAMI_REQUEST_BUFSIZE` environment variable, \
                    to a larger value (default: {}).\
                ",
                    crate::Config::default().request_bufsize
                );
                Response::RequestHeaderFieldsTooLarge()
            })?;

            if let Some(key) = RequestHeader::from_bytes(key_bytes) {
                self.headers.append(key, value);
            } else {
                self.headers
                    .insert_custom(Slice::from_bytes(key_bytes), value)
            }
        }

        if let Some(payload_size) = self.get_payload_size(
            #[cfg(feature = "__rt_native__")]
            config,
        )? {
            self.payload =
                Some(Request::read_payload(stream, r.remaining(), payload_size.get()).await?);
        }

        Ok(Some(()))
    }

    #[cfg(feature = "__rt__")]
    #[inline]
    async fn read_payload(
        stream: &mut (impl ReadableStream + Unpin),
        remaining_buf: &[u8],
        size: usize,
    ) -> Result<CowSlice, crate::Response> {
        let remaining_buf_len = remaining_buf.len();

        if remaining_buf_len == 0 || *unsafe { remaining_buf.get_unchecked(0) } == 0 {
            crate::DEBUG!("[read_payload] case: remaining_buf.is_empty() || remaining_buf[0] == 0");

            let mut bytes = vec![0; size].into_boxed_slice();
            if let Err(err) = stream.read_exact(&mut bytes).await {
                crate::ERROR!("[Request::read_payload] Failed to read payload from stream: {err}");
                return Err(crate::Response::BadRequest());
            }
            Ok(CowSlice::Own(bytes))
        } else if size <= remaining_buf_len {
            crate::DEBUG!("[read_payload] case: starts_at + size <= BUF_SIZE");

            Ok(CowSlice::Ref(unsafe {
                Slice::new_unchecked(remaining_buf.as_ptr(), size)
            }))
        } else {
            crate::DEBUG!("[read_payload] case: else");

            let mut bytes = vec![0; size].into_boxed_slice();
            let read_result = unsafe {
                // SAFETY: size > remaining_buf_len
                bytes
                    .get_unchecked_mut(..remaining_buf_len)
                    .copy_from_slice(remaining_buf);
                stream
                    .read_exact(bytes.get_unchecked_mut(remaining_buf_len..))
                    .await
            };

            if let Err(err) = read_result {
                crate::ERROR!("[Request::read_payload] Failed to read payload from stream: {err}");
                return Err(crate::Response::BadRequest());
            }
            Ok(CowSlice::Own(bytes))
        }
    }

    #[cfg(feature = "rt_worker")]
    pub(crate) async fn take_over(
        mut self: Pin<&mut Self>,
        mut req: ::worker::Request,
        env: ::worker::Env,
        ctx: ::worker::Context,
    ) -> Result<(), crate::Response> {
        self.context.load((ctx, env));

        self.method = Method::from_worker(req.method()).ok_or_else(Response::NotImplemented)?;

        {
            let url = req.url().map_err(|e| {
                crate::ERROR!("[Request::take_over] got invalid url, error: {e}");
                Response::BadRequest()
            })?;

            let path = match url.path() {
                "" => "/",
                p => p,
            };
            self.__buf__[..path.len()].copy_from_slice(path.as_bytes());
            // avoiding the immutable/mutable borrow conflict
            //
            // SAFETY:
            // - `self.__buf__` lives as long as `self`
            // - this part of `self.__buf__` is never modified after this
            let path = unsafe { std::mem::transmute::<&[u8], &[u8]>(&self.__buf__[..path.len()]) };
            self.path.init_with_request_bytes(path)?;

            if let Some(query) = url.query() {
                self.__buf__[path.len()..path.len() + query.len()]
                    .copy_from_slice(query.as_bytes());
                // avoiding the immutable/mutable borrow conflict
                //
                // SAFETY:
                // - `self.__buf__` lives as long as `self`
                // - this part of `self.__buf__` is never modified after this
                let query = unsafe {
                    std::mem::transmute::<&[u8], &[u8]>(
                        &self.__buf__[path.len()..path.len() + query.len()],
                    )
                };
                self.query = QueryParams::new(query);
            }
        }

        self.headers.take_over(req.headers());

        self.payload = Some(CowSlice::Own(
            req.bytes()
                .await
                .map_err(|_| Response::InternalServerError())?
                .into(),
        ));

        if let Some(ip) = self.headers.get("cf-connecting-ip") {
            self.ip = ip.parse().unwrap(/* We believe Cloudflare provides a valid value here... */);
        }

        Ok(())
    }

    #[cfg(feature = "rt_lambda")]
    pub(crate) fn take_over(
        mut self: Pin<&mut Self>,
        ::lambda_runtime::LambdaEvent {
            payload: req,
            context: _,
        }: ::lambda_runtime::LambdaEvent<crate::x_lambda::LambdaHTTPRequest>,
    ) -> Result<(), lambda_runtime::Error> {
        self.context.load(req.requestContext);
        {
            let path_bytes = unsafe {
                // avoiding the immutable/mutable borrow conflict
                //
                // SAFETY:
                // - `self.context` lives as long as `self`
                // - `self.context.lambda().http.path` is never modified after this
                std::mem::transmute::<&[u8], &[u8]>(self.context.lambda().http.path.as_bytes())
            };
            self.path
                .init_with_request_bytes(path_bytes)
                .map_err(|_| crate::util::ErrorMessage("unsupported path format".into()))?;
            self.method = self.context.lambda().http.method;
            self.ip = self.context.lambda().http.sourceIp;
        }

        self.__buf__[..req.rawQueryString.len()].copy_from_slice(req.rawQueryString.as_bytes());
        self.query = QueryParams::new(&self.__buf__[..req.rawQueryString.len()]);

        self.headers = req.headers;
        if !req.cookies.is_empty() {
            self.headers.set().cookie(req.cookies.join("; "));
        }

        if let Some(body) = req.body {
            self.payload = Some(CowSlice::Own(
                (if req.isBase64Encoded {
                    crate::util::base64_decode(body)?
                } else {
                    body.into_bytes()
                })
                .into_boxed_slice(),
            ));
        }

        Ok(())
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

            #[cfg(feature = "__rt__")]
            {
                d.field("ip", &self.ip);
            }

            d.field("method", &self.method)
                .field("path", &self.path.str())
                .field("queries", &self.query)
                .field("headers", &self.headers);

            if let Some(payload) = self.payload.as_ref().map(|cs| unsafe { cs.as_bytes() }) {
                d.field("payload", &String::from_utf8_lossy(payload));
            }

            d.finish()
        }
    }
};

#[cfg(feature = "__rt__")]
#[cfg(test)]
const _: () = {
    impl PartialEq for Request {
        fn eq(&self, other: &Self) -> bool {
            self.method == other.method
                && unsafe { self.path.normalized_bytes() == other.path.normalized_bytes() }
                && self.query == other.query
                && self.headers == other.headers
                && self.payload == other.payload
        }
    }
};
