#![allow(non_snake_case)]

use crate::{header::append, Fang, FangProc, Request, Response, Status};


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
#[derive(Clone)]
pub struct Cors {
    pub(crate) allow_origin:      AccessControlAllowOrigin,
    pub(crate) allow_credentials: bool,
    pub(crate) allow_methods:     Option<String>,
    pub(crate) allow_headers:     Option<String>,
    pub(crate) expose_headers:    Option<String>,
    pub(crate) max_age:           Option<u32>,
}

#[derive(Clone)]
pub(crate) enum AccessControlAllowOrigin {
    Any,
    Only(&'static str),
} impl AccessControlAllowOrigin {
    #[inline(always)] pub(crate) const fn is_any(&self) -> bool {
        match self {
            Self::Any => true,
            _ => false,
        }
    }

    #[inline(always)] pub(crate) const fn from_literal(lit: &'static str) -> Self {
        match lit.as_bytes() {
            b"*"   => Self::Any,
            origin => Self::Only(unsafe{std::str::from_utf8_unchecked(origin)}),
        }
    }

    #[inline(always)] pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::Any          => "*",
            Self::Only(origin) => origin,
        }
    }
}

impl Cors {
    /// Create `ors` fang using given `origin` as `Access-Control-Allow-Origin` header value.\
    /// (Both `"*"` and a speciffic origin are available)
    #[allow(non_snake_case)]
    pub fn new(origin: &'static str) -> Self {
        Self {
            allow_origin:      AccessControlAllowOrigin::from_literal(origin),
            allow_credentials: false,
            allow_methods:     None,
            allow_headers:     None,
            expose_headers:    None,
            max_age:           None,
        }
    }

    /* Always use default for now...
    /// Override `Access-Control-Allow-Methods` header value, it's default to
    /// all available methods on the request path.
    pub fn allow_methods<const N: usize>(mut self, methods: [Method; N]) -> Self {
        self.allow_methods = Some(methods.map(|m| m.as_str()).join(", "));
        self
    }
    */

    pub fn allow_credentials(mut self, yes: bool) -> Self {
        if yes {
            if self.allow_origin.is_any() {
                #[cfg(debug_assertions)] crate::WARNING!("\
                [WRANING] \
                'Access-Control-Allow-Origin' header \
                must not have wildcard '*' when the request's credentials mode is 'include' \
                ");
                return self
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
    type Proc = CORSProc<Inner>;
    fn chain(&self, inner: Inner) -> Self::Proc {
        CORSProc { inner, cors: self.clone() }
    }
}

pub struct CORSProc<Inner: FangProc> {
    cors:  Cors,
    inner: Inner,
}
/* Based on https://github.com/honojs/hono/blob/main/src/middleware/cors/index.ts; MIT */
impl<Inner: FangProc> FangProc for CORSProc<Inner> {
    async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
        let mut res = self.inner.bite(req).await;

        let mut h = res.headers.set();

        h = h.AccessControlAllowOrigin(self.cors.allow_origin.as_str());
        if self.cors.allow_origin.is_any() {
            h = h.Vary("Origin");
        }
        if self.cors.allow_credentials {
            h = h.AccessControlAllowCredentials("true");
        }
        if let Some(expose_headers) = &self.cors.expose_headers {
            h = h.AccessControlExposeHeaders(expose_headers.to_string());
        }

        if req.method.isOPTIONS() {
            if let Some(max_age) = self.cors.max_age {
                h = h.AccessControlMaxAge(max_age.to_string());
            }
            if let Some(allow_methods) = &self.cors.allow_methods {
                h = h.AccessControlAllowMethods(allow_methods.to_string());
            }
            if let Some(allow_headers) = self.cors.allow_headers.as_deref()
                .or_else(|| req.headers.AccessControlRequestHeaders())
            {
                h = h.AccessControlAllowHeaders(allow_headers.to_string())
                    .Vary(append("Access-Control-Request-Headers"));
            }

            /* override default `Not Implemented` response for valid preflight */
            if res.status == Status::NotImplemented {
                res.status = Status::OK;
                h.ContentType(None).ContentLength(None);
            }
        }

        #[cfg(feature="DEBUG")]
        println!("After CORS proc: res = {res:#?}");

        res
    }
}




#[cfg(debug_assertions)]
#[cfg(test)]
mod test {
    #[test] fn cors_fang_bound() {
        use crate::fang::{Fang, BoxedFPC};
        fn assert_fang<T: Fang<BoxedFPC>>() {}

        assert_fang::<super::Cors>();
    }

    #[cfg(all(feature="__rt_native__", feature="DEBUG"))]
    #[test] fn options_request() {
        use crate::prelude::*;
        use crate::testing::*;
        use super::Cors;
    
        crate::__rt__::testing::block_on(async {
            let t = Ohkami::new(
                "/hello".POST(|| async {"Hello!"})
            ).test(); {
                let req = TestRequest::OPTIONS("/");
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), Status::NotFound);
            } {
                let req = TestRequest::OPTIONS("/hello");
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), Status::NotFound);
                assert_eq!(res.text(), None);
            }

            let t = Ohkami::new((Cors::new("https://example.x.y.z"),
                "/hello".POST(|| async {"Hello!"})
            )).test(); {
                let req = TestRequest::OPTIONS("/");
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), Status::NotFound);
            } {
                let req = TestRequest::OPTIONS("/hello");
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), Status::NotFound);
                assert_eq!(res.text(), None);
            } {
                let req = TestRequest::OPTIONS("/hello")
                    .header("Access-Control-Request-Method", "DELETE");
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), Status::BadRequest/* Because `DELETE` is not available */);
                assert_eq!(res.text(), None);
            } {
                let req = TestRequest::OPTIONS("/hello")
                    .header("Access-Control-Request-Method", "POST");
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), Status::OK/* Becasue `POST` is available */);
                assert_eq!(res.text(), None);
            }
        });
    }

    #[cfg(all(feature="__rt_native__", feature="DEBUG"))]
    #[test] fn cors_headers() {
        use crate::prelude::*;
        use crate::testing::*;
        use super::Cors;
    
        crate::__rt__::testing::block_on(async {
            let t = Ohkami::new((Cors::new("https://example.example"),
                "/".GET(|| async {"Hello!"})
            )).test(); {
                let req = TestRequest::GET("/");
                let res = t.oneshot(req).await;

                assert_eq!(res.status().code(), 200);
                assert_eq!(res.text(), Some("Hello!"));

                assert_eq!(res.header("Access-Control-Allow-Origin"), Some("https://example.example"));
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
                "/abc"
                    .GET(|| async {"Hello!"})
                    .PUT(|| async {"Hello!"})
            )).test(); {
                let req = TestRequest::OPTIONS("/abc");
                let res = t.oneshot(req).await;

                assert_eq!(res.status().code(), 404/* Because `req` has no `Access-Control-Request-Method` */);
                assert_eq!(res.text(), None);

                assert_eq!(res.header("Access-Control-Allow-Origin"), Some("https://example.example"));
                assert_eq!(res.header("Access-Control-Allow-Credentials"), Some("true"));
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), None);
                assert_eq!(res.header("Access-Control-Allow-Methods"), None/* Because `req` has no `Access-Control-Request-Method` */);
                assert_eq!(res.header("Access-Control-Allow-Headers"), Some("Content-Type, X-Custom"));
                assert_eq!(res.header("Vary"), Some("Access-Control-Request-Headers"));
            } {
                let req = TestRequest::OPTIONS("/abc")
                    .header("Access-Control-Request-Method", "PUT");
                let res = t.oneshot(req).await;

                assert_eq!(res.status().code(), 200/* Because `req` HAS available `Access-Control-Request-Method` */);
                assert_eq!(res.text(), None);

                assert_eq!(res.header("Access-Control-Allow-Origin"), Some("https://example.example"));
                assert_eq!(res.header("Access-Control-Allow-Credentials"), Some("true"));
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), None);
                assert_eq!(res.header("Access-Control-Allow-Methods"), Some("GET, PUT, HEAD, OPTIONS")/* Because `req` HAS a `Access-Control-Request-Method` */);
                assert_eq!(res.header("Access-Control-Allow-Headers"), Some("Content-Type, X-Custom"));
                assert_eq!(res.header("Vary"), Some("Access-Control-Request-Headers"));
            } {
                let req = TestRequest::OPTIONS("/abc")
                    .header("Access-Control-Request-Method", "DELETE");
                let res = t.oneshot(req).await;

                assert_eq!(res.status().code(), 400/* Because `DELETE` is not available */);
                assert_eq!(res.text(), None);

                assert_eq!(res.header("Access-Control-Allow-Origin"), Some("https://example.example"));
                assert_eq!(res.header("Access-Control-Allow-Credentials"), Some("true"));
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), None);
                assert_eq!(res.header("Access-Control-Allow-Methods"), Some("GET, PUT, HEAD, OPTIONS")/* Because `req` HAS a `Access-Control-Request-Method` */);
                assert_eq!(res.header("Access-Control-Allow-Headers"), Some("Content-Type, X-Custom"));
                assert_eq!(res.header("Vary"), Some("Access-Control-Request-Headers"));
            } {
                let req = TestRequest::PUT("/abc");
                let res = t.oneshot(req).await;

                assert_eq!(res.status().code(), 200);
                assert_eq!(res.text(), Some("Hello!"));

                assert_eq!(res.header("Access-Control-Allow-Origin"), Some("https://example.example"));
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
                "/".POST(|| async {"Hello!"})
            )).test(); {
                let req = TestRequest::OPTIONS("/");
                let res = t.oneshot(req).await;

                assert_eq!(res.status().code(), 404/* Because `req` has no `Access-Control-Request-Method` */);
                assert_eq!(res.text(), None);

                assert_eq!(res.header("Access-Control-Allow-Origin"), Some("*"));
                assert_eq!(res.header("Access-Control-Allow-Credentials"), None);
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), Some("1024"));
                assert_eq!(res.header("Access-Control-Allow-Methods"), None/* Because `req` has no `Access-Control-Request-Method` */);
                assert_eq!(res.header("Access-Control-Allow-Headers"), Some("Content-Type, X-Custom"));
                assert_eq!(res.header("Vary"), Some("Origin, Access-Control-Request-Headers"));
            } {
                let req = TestRequest::OPTIONS("/")
                    .header("Access-Control-Request-Method", "POST");
                let res = t.oneshot(req).await;

                assert_eq!(res.status().code(), 200);
                assert_eq!(res.text(), None);

                assert_eq!(res.header("Access-Control-Allow-Origin"), Some("*"));
                assert_eq!(res.header("Access-Control-Allow-Credentials"), None);
                assert_eq!(res.header("Access-Control-Expose-Headers"), None);
                assert_eq!(res.header("Access-Control-Max-Age"), Some("1024"));
                assert_eq!(res.header("Access-Control-Allow-Methods"), Some("POST, OPTIONS"));
                assert_eq!(res.header("Access-Control-Allow-Headers"), Some("Content-Type, X-Custom"));
                assert_eq!(res.header("Vary"), Some("Origin, Access-Control-Request-Headers"));
            }
        });
    }
}
