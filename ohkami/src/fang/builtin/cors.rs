use crate::{Fang, FangProc, Request, Response, Status, header::append};
use std::borrow::Cow;

/// # Builtin fang for CORS config
///
/// <br>
///
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::fang::Cors;
///
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         Cors::new("https://foo.bar.org")
///             .allow_headers(["Content-Type", "X-Requested-With"])
///             .allow_credentials(true)
///             .max_age(None),
///         "/api"
///             .GET(|| async {"Hello, CORS!"}),
///     )).howl("localhost:8080").await
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Cors {
    /* pub(crate) allow_methods: Option<String>, // owe to `Handler::default_not_found()` */
    pub(crate) allow_origin: AccessControlAllowOrigin,
    pub(crate) allow_credentials: bool,
    pub(crate) allow_headers: Option<String>,
    pub(crate) expose_headers: Option<String>,
    pub(crate) max_age: Option<u32>,
}

#[derive(Clone, Debug)]
pub(crate) enum AccessControlAllowOrigin {
    Any,
    // `.access_control_allow_origin(...)` in the [`bite` impl](CorsProc::bite) requires accepts `Cow<'static, str>` so
    // it will be cheap copy if user supplies as with static string ahead of time
    Only(Cow<'static, str>),
}

impl AccessControlAllowOrigin {
    #[inline(always)]
    pub(crate) const fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    pub(crate) fn new(s: impl Into<Cow<'static, str>>) -> Result<Self, &'static str> {
        let s = s.into();
        match s.as_ref() {
            "*" => Ok(Self::Any),
            _ => super::validate_origin(&s).map(|_| Self::Only(s)),
        }
    }

    #[inline(always)]
    //This will perform expensive copy only if user provided dynamic string
    pub(crate) fn get_cow(&self) -> Cow<'static, str> {
        match self {
            Self::Any => Cow::Borrowed("*"),
            Self::Only(origin) => origin.clone(),
        }
    }
}

impl Cors {
    /// Create `Cors` fang using given `origin` as `Access-Control-Allow-Origin` header value.\
    /// (Both `"*"` and a specific origin are available)
    pub fn new(origin: impl Into<Cow<'static, str>>) -> Self {
        Self {
            allow_origin: AccessControlAllowOrigin::new(origin)
                .unwrap_or_else(|err| panic!("[Cors::new] {err}")),
            allow_credentials: false,
            allow_headers: None,
            expose_headers: None,
            max_age: None,
        }
    }

    #[inline]
    /// Creates `Cors` with any origin allowed
    pub const fn any() -> Self {
        Self {
            allow_origin: AccessControlAllowOrigin::Any,
            allow_credentials: false,
            allow_headers: None,
            expose_headers: None,
            max_age: None,
        }
    }

    pub fn allow_credentials(mut self, yes: bool) -> Self {
        if yes {
            if self.allow_origin.is_any() {
                #[cfg(debug_assertions)]
                {
                    crate::WARNING!(
                        "\
                        'Access-Control-Allow-Origin' header \
                        must not have wildcard '*' when the request's credentials mode is 'include' \
                    "
                    );
                }
                return self;
            }
            self.allow_credentials = true;
        } else {
            self.allow_credentials = false;
        }
        self
    }
    pub fn allow_headers<const N: usize>(mut self, headers: [&'static str; N]) -> Self {
        self.allow_headers = (!headers.is_empty()).then_some(headers.join(", "));
        self
    }
    pub fn expose_headers<const N: usize>(mut self, headers: [&'static str; N]) -> Self {
        self.expose_headers = (!headers.is_empty()).then_some(headers.join(", "));
        self
    }
    pub fn max_age(mut self, delta_seconds: Option<u32>) -> Self {
        self.max_age = delta_seconds;
        self
    }
    pub fn verify_origin<'a>(origin: &'a str, allow_origin: Cow<'a, str>) -> Cow<'a ,str> {
        //Check protocol being the same and character count being within limit, if not return.
        let Some((protocol, rest)) = origin.split_once("://") else {
            return allow_origin;
        };
        let Some((allow_protocol, allow_rest)) = allow_origin.split_once("://") else {
            return allow_origin;
        };

        if protocol == allow_protocol && origin.chars().count() <= 253 {
            let (allow_host, allow_port) = allow_rest
                .split_once(':')
                .map_or((allow_rest, None), |(h, p)| (h, Some(p)));

            //No wildcards in Cors at all, return default
            if !allow_host.starts_with("*.") && allow_port.is_some_and(|p| p != "*") {
                return allow_origin;
            }

            let (host, port) = rest
                .split_once(':')
                .map_or((rest, None), |(h, p)| (h, Some(p)));
            //If no port wildcard in Cors, enforce similarity
            if allow_port.is_some_and(|p| p != "*") {
                if port != allow_port {
                    return allow_origin;
                }
            }

            if !allow_host.starts_with("*.") {
                if host != allow_host {
                    return allow_origin
                }
            }

            //Port must be in range of u16, and must be either * or a string of numbers.
            if port.is_some_and(|p| p.parse::<u16>().is_err()) {
                return allow_origin;
            }

            //Origin host must not be empty and only contain up to 63 characters from a-Z, 0-9, '-', '*'.
            if !host.split('.').all(|part| {
                !part.is_empty()
                    && part.chars().all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-')
                    && part.chars().count() <= 63)
            }) {
                return allow_origin;
            }

            let Some((subdomain, sld)) = host.split_once('.') else {
                return allow_origin;
            };
            let Some((allow_subdomain, allow_sld)) = allow_host.split_once('.') else {
                return allow_origin;
            };

            //The latter parts of the host must exactly match, as there's no allowed wildcards here.
            if sld != allow_sld {
                return allow_origin;
            }

            //If the request is from an IP address, which cannot have a subdomain, and there's a port wildcard, return origin, otherwise default.
            if sld.split('.').all(|part| part.chars().all(|c| c.is_numeric())) {
                return if allow_port.is_some_and(|p| p == "*") && subdomain != "*" {
                    Cow::Borrowed(origin)
                } else {
                    allow_origin
                }
            }

            //If subdomain is only alphanumeric characters and there's no wildcard, the subdomain must exactly match.
            if subdomain.chars().all(|c| c.is_ascii_alphanumeric()) {
                if allow_subdomain != "*" {
                    if subdomain != allow_subdomain {
                        return allow_origin;
                    }
                    if allow_port.is_some_and(|p| p == "*") {
                        //Subdomain is valid, and port doesn't matter, return request origin
                        return Cow::Borrowed(origin)
                    }
                } else if allow_port.is_some_and(|p| p != "*") {
                    return if port == allow_port {
                        //Subdomain is wildcard, port is valid
                        Cow::Borrowed(origin)
                    } else {
                        allow_origin
                    };
                } else {
                    //Subdomain is a wildcard and so is the port
                    return Cow::Borrowed(origin)
                }
            }
        }
        allow_origin //No wildcards
    }
}

impl<Inner: FangProc> Fang<Inner> for Cors {
    type Proc = CorsProc<Inner>;
    fn chain(&self, inner: Inner) -> Self::Proc {
        CorsProc {
            inner,
            cors: self.clone(),
        }
    }
}

pub struct CorsProc<Inner: FangProc> {
    cors: Cors,
    inner: Inner,
}
/* Based on https://github.com/honojs/hono/blob/main/src/middleware/cors/index.ts; MIT */
impl<Inner: FangProc> FangProc for CorsProc<Inner> {
    async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
        let mut res = self.inner.bite(req).await;
        let allow_origin = Cors::verify_origin(req.headers.origin().unwrap_or_else(|| ""), self.cors.allow_origin.get_cow()).into_owned();

        res.headers
            .set()
            .access_control_allow_origin(allow_origin)
            .vary(self.cors.allow_origin.is_any().then_some("Origin".into()))
            .access_control_allow_credentials(self.cors.allow_credentials.then_some("true".into()))
            .access_control_expose_headers(
                self.cors
                    .expose_headers
                    .as_ref()
                    .map(|s| s.to_string().into()),
            );
        if req.method.isOPTIONS() {
            res.headers
                .set()
                .access_control_max_age(self.cors.max_age.map(|v| v.to_string().into()));
            if let Some(allow_headers) = self.cors.allow_headers.as_ref()
                && !allow_headers.is_empty()
            {
                res.headers
                    .set()
                    .access_control_allow_headers(allow_headers.to_string())
                    .vary(append("Access-Control-Request-Headers"));
            }
            if res.status == Status::NotImplemented {
                // override default `NotImplemented` response for valid preflight.
                // see `Handler::default_not_found()`.
                res.status = Status::OK;
                res.headers.set().content_type(None).content_length(None);
            }
        }

        crate::DEBUG!("After CORS proc: res = {res:#?}");

        res
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn cors_accept_regular_ip() {
        assert_eq!("https://192.168.1.41:5173", super::Cors::verify_origin("https://192.168.1.41:5173", std::borrow::Cow::Borrowed("https://192.168.1.41:5173")))
    }

    #[test]
    fn cors_accept_regular_domain() {
        assert_eq!("https://example.com", super::Cors::verify_origin("https://example.com", std::borrow::Cow::Borrowed("https://example.com")));
        assert_eq!("https://sub.example.com", super::Cors::verify_origin("https://sub.example.com", std::borrow::Cow::Borrowed("https://sub.example.com")))
    }

    #[test]
    fn cors_accept_wildcard_in_ip_port() {
        assert_eq!("https://192.168.1.41:5173", super::Cors::verify_origin("https://192.168.1.41:5173", std::borrow::Cow::Borrowed("https://192.168.1.41:*")))
    }

    #[test]
    fn cors_accept_wildcard_in_port() {
        assert_eq!("https://example.com:5173", super::Cors::verify_origin("https://example.com:5173", std::borrow::Cow::Borrowed("https://example.com:*")))
    }

    #[test]
    fn cors_accept_wildcard_in_subdomain() {
        assert_eq!("https://test.example.com", super::Cors::verify_origin("https://test.example.com", std::borrow::Cow::Borrowed("https://*.example.com")))
    }

    #[test]
    fn cors_deny_wildcard_in_ip_subdomain() {
        assert_eq!("https://192.168.1.0:8080", super::Cors::verify_origin("https://192.*.1.0:8080", std::borrow::Cow::Borrowed("https://192.168.1.0:8080")))
    }

    #[test]
    fn cors_deny_wildcard_in_sld() {
        assert_eq!("https://test.example.com:8080", super::Cors::verify_origin("https://test.*.com:8080", std::borrow::Cow::Borrowed("https://test.example.com:8080")))
    }

    #[test]
    fn cors_deny_invalid_ip() {
        assert_eq!("https://192.168.1.0:8080", super::Cors::verify_origin("https://192.168.a.0:8080", std::borrow::Cow::Borrowed("https://192.168.1.0:8080")))
    }

    #[test]
    fn cors_deny_invalid_ip_port_range() {
        assert_eq!("https://192.168.1.0:8080", super::Cors::verify_origin("https://192.168.1.0:80080", std::borrow::Cow::Borrowed("https://192.168.1.0:8080")))
    }

    #[test]
    fn cors_new_with_str_or_string() {
        let _: super::Cors = super::Cors::new("https://example.com");
        let _: super::Cors = super::Cors::new(String::from("https://") + "example.com");
    }

    #[test]
    fn cors_wildcard_validation() {
        let _: super::Cors = super::Cors::new("https://*.example.com");
        let _: super::Cors = super::Cors::new("https://example.com:*");
        let _: super::Cors = super::Cors::new("https://*.example.com:*");
        let _: super::Cors = super::Cors::new("http://123example.com");
        let _: super::Cors = super::Cors::new("https://abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcde.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijk.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijk.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijk.com");
    }

    #[test]
    #[should_panic(expected = "invalid origin: 'http' or 'https' scheme is required at the start of the string.")]
    fn cors_scheme_invalidation() {
        let _: super::Cors = super::Cors::new("foobarhttp://example.com");
    }

    #[test]
    #[should_panic(expected = "invalid origin: maximum length 253 for domain exceeded.")]
    fn cors_length_invalidation() {
        let _: super::Cors = super::Cors::new("https://abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcde.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijk.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijk.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijkl.com");
    }

    #[test]
    #[should_panic(expected = "invalid origin: invalid host.")]
    fn cors_part_length_invalidation() {
        let _: super::Cors = super::Cors::new("https://www.abcdefghijklmnopqrstuvwxyzabcdefghijklmnoqrstuvwxyzabcdefghijklmnopqrstuvwxyz.com");
    }

    #[test]
    #[should_panic(expected = "[Cors::new] invalid origin: port must be a number between 0 and 65535 or wildcard '*'.")]
    fn cors_port_invalidation() {
        let _: super::Cors = super::Cors::new("http://example.com:abcd");
    }

    #[test]
    #[should_panic(expected = "invalid origin: host must start with an alphanumeric character or wildcard '*'.")]
    fn cors_host_invalidation() {
        let _: super::Cors = super::Cors::new("http://%example.com");
    }

    #[test]
    fn cors_fang_bound() {
        use crate::fang::{BoxedFPC, Fang};
        fn assert_fang<T: Fang<BoxedFPC>>() {}

        assert_fang::<super::Cors>();
    }

    #[cfg(all(feature = "__rt_native__", feature = "DEBUG"))]
    #[test]
    fn options_request() {
        use super::Cors;
        use crate::prelude::*;
        use crate::testing::*;

        crate::__rt__::testing::block_on(async {
            let t = Ohkami::new("/hello".POST(|| async { "Hello!" })).test();
            {
                let req = TestRequest::OPTIONS("/");
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), Status::NotFound);
            }
            {
                let req = TestRequest::OPTIONS("/hello");
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), Status::NotFound);
                assert_eq!(res.text(), None);
            }

            let t = Ohkami::new((
                Cors::new("https://example.x.y.z"),
                "/hello".POST(|| async { "Hello!" }),
            ))
            .test();
            {
                let req = TestRequest::OPTIONS("/");
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), Status::NotFound);
            }
            {
                let req = TestRequest::OPTIONS("/hello");
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), Status::NotFound);
                assert_eq!(res.text(), None);
            }
            {
                let req = TestRequest::OPTIONS("/hello")
                    .header("Access-Control-Request-Method", "DELETE");
                let res = t.oneshot(req).await;
                assert_eq!(
                    res.status(),
                    Status::BadRequest /* Because `DELETE` is not available */
                );
                assert_eq!(res.text(), None);
            }
            {
                let req =
                    TestRequest::OPTIONS("/hello").header("Access-Control-Request-Method", "POST");
                let res = t.oneshot(req).await;
                assert_eq!(
                    res.status(),
                    Status::OK /* Becasue `POST` is available */
                );
                assert_eq!(res.text(), None);
            }
        });
    }

    #[cfg(all(feature = "__rt_native__", feature = "DEBUG"))]
    #[test]
    fn cors_headers() {
        use super::Cors;
        use crate::prelude::*;
        use crate::testing::*;

        crate::__rt__::testing::block_on(async {
            let t = Ohkami::new((
                Cors::new("https://example.example"),
                "/".GET(|| async { "Hello!" }),
            ))
            .test();
            {
                let req = TestRequest::GET("/");
                let res = t.oneshot(req).await;

                assert_eq!(res.status().code(), 200);
                assert_eq!(res.text(), Some("Hello!"));

                assert_eq!(
                    res.header("Access-Control-Allow-Origin"),
                    Some("https://example.example")
                );
                assert_eq!(res.header("Access-Control-Allow-Credentials"), None);
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), None);
                assert_eq!(res.header("Access-Control-Allow-Methods"), None);
                assert_eq!(res.header("Access-Control-Allow-Headers"), None);
                assert_eq!(res.header("Vary"), None);
            }

            let t = Ohkami::new((
                Cors::new("https://example.example")
                    .allow_credentials(true)
                    .allow_headers(["Content-Type", "X-Custom"]),
                "/abc".GET(|| async { "Hello!" }).PUT(|| async { "Hello!" }),
            ))
            .test();
            {
                let req = TestRequest::OPTIONS("/abc");
                let res = t.oneshot(req).await;

                assert_eq!(
                    res.status().code(),
                    404 /* Because `req` has no `Access-Control-Request-Method` */
                );
                assert_eq!(res.text(), None);

                assert_eq!(
                    res.header("Access-Control-Allow-Origin"),
                    Some("https://example.example")
                );
                assert_eq!(res.header("Access-Control-Allow-Credentials"), Some("true"));
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), None);
                assert_eq!(
                    res.header("Access-Control-Allow-Methods"),
                    None /* Because `req` has no `Access-Control-Request-Method` */
                );
                assert_eq!(
                    res.header("Access-Control-Allow-Headers"),
                    Some("Content-Type, X-Custom")
                );
                assert_eq!(res.header("Vary"), Some("Access-Control-Request-Headers"));
            }
            {
                let req =
                    TestRequest::OPTIONS("/abc").header("Access-Control-Request-Method", "PUT");
                let res = t.oneshot(req).await;

                assert_eq!(
                    res.status().code(),
                    200 /* Because `req` HAS available `Access-Control-Request-Method` */
                );
                assert_eq!(res.text(), None);

                assert_eq!(
                    res.header("Access-Control-Allow-Origin"),
                    Some("https://example.example")
                );
                assert_eq!(res.header("Access-Control-Allow-Credentials"), Some("true"));
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), None);
                assert_eq!(
                    res.header("Access-Control-Allow-Methods"),
                    Some("GET, PUT, HEAD, OPTIONS") /* Because `req` HAS a `Access-Control-Request-Method` */
                );
                assert_eq!(
                    res.header("Access-Control-Allow-Headers"),
                    Some("Content-Type, X-Custom")
                );
                assert_eq!(res.header("Vary"), Some("Access-Control-Request-Headers"));
            }
            {
                let req =
                    TestRequest::OPTIONS("/abc").header("Access-Control-Request-Method", "DELETE");
                let res = t.oneshot(req).await;

                assert_eq!(
                    res.status().code(),
                    400 /* Because `DELETE` is not available */
                );
                assert_eq!(res.text(), None);

                assert_eq!(
                    res.header("Access-Control-Allow-Origin"),
                    Some("https://example.example")
                );
                assert_eq!(res.header("Access-Control-Allow-Credentials"), Some("true"));
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), None);
                assert_eq!(
                    res.header("Access-Control-Allow-Methods"),
                    Some("GET, PUT, HEAD, OPTIONS") /* Because `req` HAS a `Access-Control-Request-Method` */
                );
                assert_eq!(
                    res.header("Access-Control-Allow-Headers"),
                    Some("Content-Type, X-Custom")
                );
                assert_eq!(res.header("Vary"), Some("Access-Control-Request-Headers"));
            }
            {
                let req = TestRequest::PUT("/abc");
                let res = t.oneshot(req).await;

                assert_eq!(res.status().code(), 200);
                assert_eq!(res.text(), Some("Hello!"));

                assert_eq!(
                    res.header("Access-Control-Allow-Origin"),
                    Some("https://example.example")
                );
                assert_eq!(res.header("Access-Control-Allow-Credentials"), Some("true"));
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), None);
                assert_eq!(res.header("Access-Control-Allow-Methods"), None);
                assert_eq!(res.header("Access-Control-Allow-Headers"), None);
                assert_eq!(res.header("Vary"), None);
            }

            let t = Ohkami::new((
                Cors::new("*")
                    .allow_headers(["Content-Type", "X-Custom"])
                    .max_age(Some(1024)),
                "/".POST(|| async { "Hello!" }),
            ))
            .test();
            {
                let req = TestRequest::OPTIONS("/");
                let res = t.oneshot(req).await;

                assert_eq!(
                    res.status().code(),
                    404 /* Because `req` has no `Access-Control-Request-Method` */
                );
                assert_eq!(res.text(), None);

                assert_eq!(res.header("Access-Control-Allow-Origin"), Some("*"));
                assert_eq!(res.header("Access-Control-Allow-Credentials"), None);
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), Some("1024"));
                assert_eq!(
                    res.header("Access-Control-Allow-Methods"),
                    None /* Because `req` has no `Access-Control-Request-Method` */
                );
                assert_eq!(
                    res.header("Access-Control-Allow-Headers"),
                    Some("Content-Type, X-Custom")
                );
                assert_eq!(
                    res.header("Vary"),
                    Some("Origin, Access-Control-Request-Headers")
                );
            }
            {
                let req = TestRequest::OPTIONS("/").header("Access-Control-Request-Method", "POST");
                let res = t.oneshot(req).await;

                assert_eq!(res.status().code(), 200);
                assert_eq!(res.text(), None);

                assert_eq!(res.header("Access-Control-Allow-Origin"), Some("*"));
                assert_eq!(res.header("Access-Control-Allow-Credentials"), None);
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), Some("1024"));
                assert_eq!(
                    res.header("Access-Control-Allow-Methods"),
                    Some("POST, OPTIONS")
                );
                assert_eq!(
                    res.header("Access-Control-Allow-Headers"),
                    Some("Content-Type, X-Custom")
                );
                assert_eq!(
                    res.header("Vary"),
                    Some("Origin, Access-Control-Request-Headers")
                );
            }
        });
    }
}
