#![allow(non_snake_case)]

use crate::{header::append, Fang, FangProc, Request, Response, Status};


/// # Builtin fang for CORS config
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::fang::CORS;
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         CORS::new("https://foo.bar.org")
///             .AllowHeaders(["Content-Type", "X-Requested-With"])
///             .AllowCredentials()
///             .MaxAge(86400),
///         "/api"
///             .GET(|| async {"Hello, CORS!"}),
///     )).howl("localhost:8080").await
/// }
/// ```
#[derive(Clone)]
pub struct CORS {
    pub(crate) AllowOrigin:      AccessControlAllowOrigin,
    pub(crate) AllowCredentials: bool,
    pub(crate) AllowMethods:     Option<String>,
    pub(crate) AllowHeaders:     Option<String>,
    pub(crate) ExposeHeaders:    Option<String>,
    pub(crate) MaxAge:           Option<u32>,
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

impl CORS {
    /// Create `CORS` fang using given `AllowOrigin` as `Access-Control-Allow-Origin` header value.\
    /// (Both `"*"` and a speciffic origin are available)
    #[allow(non_snake_case)]
    pub fn new(AllowOrigin: &'static str) -> Self {
        Self {
            AllowOrigin:      AccessControlAllowOrigin::from_literal(AllowOrigin),
            AllowCredentials: false,
            AllowMethods:     None,
            AllowHeaders:     None,
            ExposeHeaders:    None,
            MaxAge:           None,
        }
    }

    /* Always use default for now...
    /// Override `Access-Control-Allow-Methods` header value, it's default to
    /// all available methods on the request path.
    pub fn AllowMethods<const N: usize>(mut self, methods: [Method; N]) -> Self {
        self.AllowMethods = Some(methods.map(|m| m.as_str()).join(", "));
        self
    }
    */

    pub fn AllowCredentials(mut self) -> Self {
        if self.AllowOrigin.is_any() {
            #[cfg(debug_assertions)] crate::WARNING!("\
                [WRANING] \
                'Access-Control-Allow-Origin' header \
                must not have wildcard '*' when the request's credentials mode is 'include' \
            ");
            return self
        }
        self.AllowCredentials = true;
        self
    }
    pub fn AllowHeaders<const N: usize>(mut self, headers: [&'static str; N]) -> Self {
        self.AllowHeaders = Some(headers.join(", "));
        self
    }
    pub fn ExposeHeaders<const N: usize>(mut self, headers: [&'static str; N]) -> Self {
        self.ExposeHeaders = Some(headers.join(", "));
        self
    }
    pub fn MaxAge(mut self, delta_seconds: u32) -> Self {
        self.MaxAge = Some(delta_seconds);
        self
    }
}

impl<Inner: FangProc> Fang<Inner> for CORS {
    type Proc = CORSProc<Inner>;
    fn chain(&self, inner: Inner) -> Self::Proc {
        CORSProc { inner, cors: self.clone() }
    }
}

pub struct CORSProc<Inner: FangProc> {
    cors:  CORS,
    inner: Inner,
}
/* Based on https://github.com/honojs/hono/blob/main/src/middleware/cors/index.ts; MIT */
impl<Inner: FangProc> FangProc for CORSProc<Inner> {
    async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
        let mut res = self.inner.bite(req).await;

        let mut h = res.headers.set();

        h = h.AccessControlAllowOrigin(self.cors.AllowOrigin.as_str());
        if self.cors.AllowOrigin.is_any() {
            h = h.Vary("Origin");
        }
        if self.cors.AllowCredentials {
            h = h.AccessControlAllowCredentials("true");
        }
        if let Some(expose_headers) = &self.cors.ExposeHeaders {
            h = h.AccessControlExposeHeaders(expose_headers.to_string());
        }

        if req.method.isOPTIONS() {
            if let Some(max_age) = self.cors.MaxAge {
                h = h.AccessControlMaxAge(max_age.to_string());
            }
            if let Some(allow_methods) = &self.cors.AllowMethods {
                h = h.AccessControlAllowMethods(allow_methods.to_string());
            }
            if let Some(allow_headers) = self.cors.AllowHeaders.as_deref()
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

        assert_fang::<super::CORS>();
    }

    #[cfg(all(feature="__rt_native__", feature="DEBUG"))]
    #[test] fn options_request() {
        use crate::prelude::*;
        use crate::testing::*;
        use super::CORS;
    
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

            let t = Ohkami::new((CORS::new("https://example.x.y.z"),
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
        use super::CORS;
    
        crate::__rt__::testing::block_on(async {
            let t = Ohkami::new((CORS::new("https://example.example"),
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
                CORS::new("https://example.example")
                    .AllowCredentials()
                    .AllowHeaders(["Content-Type", "X-Custom"]),
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
                CORS::new("*")
                    .AllowHeaders(["Content-Type", "X-Custom"])
                    .MaxAge(1024),
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
