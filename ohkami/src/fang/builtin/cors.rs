use super::{Origin, OriginError};
use crate::{Fang, FangProc, Request, Response, Status, header::append};
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub enum AllowOriginConfig {
    CorsOrigin(CorsOrigin),
    Any,
}

#[derive(Clone, Debug)]
pub struct CorsOrigin {
    base_origin: Origin,
    any_port: bool,
    any_subdomain: bool,
}

#[derive(Debug, PartialEq)]
pub enum CorsOriginError {
    InvalidOrigin(OriginError),
    FaultyWildcardInIp,
}

impl CorsOrigin {
    /// Parse string into [`CorsOrigin`], checks for wildcards inside the string,
    /// and if all appropriate validation succeeds inside [`Origin`] and [`CorsOrigin`] returns Self.
    ///
    /// # Examples
    /// ```ignore
    /// fn run() {
    ///     CorsOrigin::new("https://*.localhost:3000").unwrap(); //Gives CorsOrigin
    ///     CorsOrigin::new("https://*.localhost:").unwrap(); //Gives CorsOriginError
    /// }
    /// ```
    /// # Errors
    /// This function returns an error if the given string fails either of the following criteria:
    /// - If an IP-address is using a subdomain wildcard.
    /// - If the validation included in [`Origin`] fails.
    ///
    fn new(s: &str) -> Result<Self, CorsOriginError> {
        let mut any_port = false;
        let mut any_subdomain = false;

        let mut s = Cow::Borrowed(s);

        if let Some(rest) = s.strip_suffix(":*") {
            any_port = true;
            s = Cow::Owned(rest.to_string());
        }

        if let Some((scheme @ ("http://" | "https://"), host)) = s.split_once("*.") {
            if let Some((ip, _port)) = host.rsplit_once(":")
                && ip.parse::<core::net::IpAddr>().is_ok()
            {
                return Err(CorsOriginError::FaultyWildcardInIp);
            }

            any_subdomain = true;
            s = Cow::Owned(scheme.to_string() + host);
        }

        let base_origin = Origin::new(&s).map_err(CorsOriginError::InvalidOrigin)?;

        Ok(Self {
            base_origin,
            any_port,
            any_subdomain,
        })
    }
}

impl std::fmt::Display for CorsOriginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            CorsOriginError::InvalidOrigin(e) => e.to_string(),
            CorsOriginError::FaultyWildcardInIp => {
                String::from("Found invalid wildcard in IP address.")
            }
        };
        write!(f, "{}", output)
    }
}

impl AllowOriginConfig {
    /// Returns a fallback Access-Control-Allow-Origin String of this [`AllowOriginConfig`].
    fn fallback_access_control_allow_origin(&self) -> String {
        match &self {
            AllowOriginConfig::Any => String::from("*"),
            // `base_origin` itself is always an allowed origin.
            AllowOriginConfig::CorsOrigin(cors_origin) => cors_origin.base_origin.to_string(),
        }
    }

    /// Parse string based on the Cors origin string syntax.
    ///
    /// # Examples
    /// ```ignore
    /// fn run() {
    ///     AllowOriginConfig::new("https://localhost:3000").unwrap(); //Gives AllowOriginConfig::CorsOrigin
    ///     AllowOriginConfig::new("*").unwrap(); //Gives AllowOriginConfig::Any
    /// }
    /// ```
    /// # Errors
    /// This function will return an error if the given URI string fails the validation included in [`CorsOrigin`] or [`Origin`].
    ///
    fn new(s: &str) -> Result<Self, CorsOriginError> {
        match s {
            "*" => Ok(Self::Any),
            _ => CorsOrigin::new(s).map(Self::CorsOrigin),
        }
    }

    /// Checks if according to the noted rules for wildcards in this struct, the incoming origin would match.
    ///
    /// # Examples
    /// ```ignore
    /// fn run() {
    ///     let cors = AllowOriginConfig::new("*").unwrap();
    ///     let origin = Origin::new("https://localhost:5173").unwrap();
    ///     assert!(cors.allows(origin)); // true
    /// }
    /// ```
    ///
    fn allows(&self, incoming_origin: &Origin) -> bool {
        match self {
            AllowOriginConfig::CorsOrigin(cors_origin) => {
                if !(cors_origin.base_origin.scheme() == incoming_origin.scheme()) {
                    return false;
                }
                // If no port wildcard, check if ports align.
                if !cors_origin.any_port && cors_origin.base_origin.port() != incoming_origin.port()
                {
                    return false;
                }

                // If we do not support any subdomain, we can just fully compare the two, as no additional validation is necessary.
                if !cors_origin.any_subdomain {
                    return cors_origin.base_origin.host() == incoming_origin.host();
                }

                if cors_origin.base_origin.host() == incoming_origin.host() {
                    true
                } else {
                    //Check if the options don't already align
                    incoming_origin
                        .host()
                        .split_once('.')
                        .is_some_and(|(_, h)| h == cors_origin.base_origin.host())
                }
            }
            AllowOriginConfig::Any => {
                // Anything goes
                true
            }
        }
    }

    /// Returns if this [`AllowOriginConfig`] is [`AllowOriginConfig::Any`].
    fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }
}

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
    pub(crate) allow_origin_config: AllowOriginConfig,
    pub(crate) allow_credentials: bool,
    pub(crate) allow_headers: Option<String>,
    pub(crate) expose_headers: Option<String>,
    pub(crate) max_age: Option<u32>,
}

impl Cors {
    /// Create `Cors` fang using given `origin` as `Access-Control-Allow-Origin` header value.\
    /// (Both `"*"` and a specific origin are available)
    pub fn new(origin: impl Into<Cow<'static, str>>) -> Self {
        Self {
            allow_origin_config: AllowOriginConfig::new(origin.into().as_ref())
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
            allow_origin_config: AllowOriginConfig::Any,
            allow_credentials: false,
            allow_headers: None,
            expose_headers: None,
            max_age: None,
        }
    }

    pub fn allow_credentials(mut self, yes: bool) -> Self {
        if yes {
            if self.allow_origin_config.is_any() {
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
        let incoming_origin = req.headers.origin().and_then(|s| Origin::new(s).ok());
        let access_control_allow_origin = match incoming_origin {
            Some(incoming) if self.cors.allow_origin_config.allows(&incoming) => {
                incoming.to_string()
            }
            _ => self
                .cors
                .allow_origin_config
                .fallback_access_control_allow_origin(),
        };

        res.headers
            .set()
            .access_control_allow_origin(access_control_allow_origin)
            .vary(
                self.cors
                    .allow_origin_config
                    .is_any()
                    .then_some("Origin".into()),
            )
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
    use super::{AllowOriginConfig, CorsOriginError, Origin};
    use crate::fang::OriginError;

    #[test]
    fn cors_accept_regular_origin_ip() {
        assert!(
            AllowOriginConfig::new("https://192.168.1.41:5173")
                .unwrap()
                .allows(&Origin::new("https://192.168.1.41:5173").unwrap())
        )
    }

    #[test]
    fn cors_accept_regular_origin_domain() {
        assert!(
            AllowOriginConfig::new("https://example.com")
                .unwrap()
                .allows(&Origin::new("https://example.com").unwrap())
        );
        assert!(
            AllowOriginConfig::new("https://example.co.uk")
                .unwrap()
                .allows(&Origin::new("https://example.co.uk").unwrap())
        );
        assert!(
            AllowOriginConfig::new("https://sub.example.com")
                .unwrap()
                .allows(&Origin::new("https://sub.example.com").unwrap())
        );
        assert!(
            AllowOriginConfig::new("https://sub.example.co.uk")
                .unwrap()
                .allows(&Origin::new("https://sub.example.co.uk").unwrap())
        );
    }

    #[test]
    fn cors_accept_origin_localhost() {
        assert!(
            AllowOriginConfig::new("https://localhost:5173")
                .unwrap()
                .allows(&Origin::new("https://localhost:5173").unwrap())
        );
        assert!(
            AllowOriginConfig::new("https://localhost:*")
                .unwrap()
                .allows(&Origin::new("https://localhost:5173").unwrap())
        );
    }

    #[test]
    fn cors_accept_wildcard_match_in_own_ip_port() {
        assert!(
            AllowOriginConfig::new("https://192.168.1.2:*")
                .unwrap()
                .allows(&Origin::new("https://192.168.1.2:5173").unwrap())
        );
    }

    #[test]
    fn cors_accept_wildcard_match_in_own_port() {
        assert!(
            AllowOriginConfig::new("https://example.com:*")
                .unwrap()
                .allows(&Origin::new("https://example.com:5173").unwrap())
        );
        assert!(
            AllowOriginConfig::new("https://example.co.uk:*")
                .unwrap()
                .allows(&Origin::new("https://example.co.uk:5173").unwrap())
        );
    }

    #[test]
    fn cors_accept_wildcard_match_in_own_subdomain() {
        assert!(
            AllowOriginConfig::new("https://*.example.com")
                .unwrap()
                .allows(&Origin::new("https://test.example.com").unwrap())
        );
        assert!(
            AllowOriginConfig::new("https://*.example.co.uk")
                .unwrap()
                .allows(&Origin::new("https://test.example.co.uk").unwrap())
        );
    }

    #[test]
    fn cors_deny_indirect_origin_subdomain() {
        assert!(
            !AllowOriginConfig::new("https://*.example.com")
                .unwrap()
                .allows(&Origin::new("https://a.b.example.com").unwrap())
        );
    }

    #[test]
    fn cors_deny_wrong_origin_subdomain() {
        assert!(
            !AllowOriginConfig::new("https://b.example.com")
                .unwrap()
                .allows(&Origin::new("https://a.b.example.com").unwrap())
        );
    }

    #[test]
    fn cors_deny_wrong_domain() {
        assert!(
            !AllowOriginConfig::new("https://example.com")
                .unwrap()
                .allows(&Origin::new("https://anexample.com").unwrap())
        );
        assert!(
            !AllowOriginConfig::new("https://example.co.uk")
                .unwrap()
                .allows(&Origin::new("https://example.uk").unwrap())
        );
        assert!(
            !AllowOriginConfig::new("https://example.co.uk")
                .unwrap()
                .allows(&Origin::new("https://example.co").unwrap())
        );
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
        let _: super::Cors = super::Cors::new(
            "https://abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcde.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijk.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijk.abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijk.com",
        );
    }

    #[test]
    fn cors_ip_subdomain_wildcard_invalidation() {
        assert_eq!(
            AllowOriginConfig::new("https://*.168.1.0:8080").unwrap_err(),
            CorsOriginError::InvalidOrigin(OriginError::FaultyIp)
        );
        assert_eq!(
            AllowOriginConfig::new("https://*.192.168.1.0:8080").unwrap_err(),
            CorsOriginError::FaultyWildcardInIp
        )
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
