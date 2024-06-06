#![allow(non_snake_case)]

use crate::{header::append, Fang, FangProc, IntoResponse, Method, Request, Response, Status};


/// # Builtin fang for CORS config
/// 
/// <br>
/// 
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::builtin::fang::CORS;
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::with((
///         CORS::new("https://foo.bar.org")
///             .AllowMethods([Method::GET, Method::POST])
///             .AllowHeaders(["Content-Type", "X-Requested-With"])
///             .AllowCredentials()
///             .MaxAge(86400),
///     ), (
///         "/api".GET(|| async {
///             "Hello, CORS!"
///         }),
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
            AllowMethods:     Some(String::from("GET, PUT, POST, PATCH, DELETE, OPTIONS, HEAD")),
            AllowHeaders:     None,
            ExposeHeaders:    None,
            MaxAge:           None,
        }
    }

    pub fn AllowCredentials(mut self) -> Self {
        if self.AllowOrigin.is_any() {
            #[cfg(debug_assertions)] crate::warning!("\
                [WRANING] \
                'Access-Control-Allow-Origin' header \
                must not have wildcard '*' when the request's credentials mode is 'include' \
            ");

            return self
        }

        self.AllowCredentials = true;
        self
    }
    pub fn AllowMethods<const N: usize>(self, methods: [Method; N]) -> Self {
        Self {
            AllowMethods: Some(methods.map(|m| m.as_str())
                .join(", ")
            ), ..self
        }
    }
    pub fn AllowHeaders<const N: usize>(self, headers: [&'static str; N]) -> Self {
        Self {
            AllowHeaders: Some(headers
                .join(", ")
            ), ..self
        }
    }
    pub fn ExposeHeaders<const N: usize>(self, headers: [&'static str; N]) -> Self {
        Self {
            ExposeHeaders: Some(headers
                .join(", ")
            ), ..self
        }
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
        let mut res = self.inner.bite(req).await.into_response();

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

            if res.status.code() < 300 {
                h.ContentType(None).ContentLength(None);
                res.status = Status::OK;
            }
        }

        #[cfg(feature="DEBUG")]
        println!("After CORS proc: res = {res:#?}");

        res
    }
}




#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
#[cfg(feature="testing")]
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::testing::*;
    use super::CORS;

    #[crate::__rt__::test] async fn options_request() {
        let t = Ohkami::with(
            CORS::new("https://example.x.y.z"),
            "/hello".GET(|| async {"Hello!"})
        ).test(); {
            let req = TestRequest::OPTIONS("/");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::NotFound);
        } {
            let req = TestRequest::OPTIONS("/hello");
            let res = t.oneshot(req).await;
            assert_eq!(res.status(), Status::OK);
            assert_eq!(res.text(), None);
        }
    }

    #[crate::__rt__::test] async fn cors_headers() {
        let t = Ohkami::with(
            CORS::new("https://example.example"),
            "/".GET(|| async {"Hello!"})
        ).test(); {
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

        let t = Ohkami::with(
            CORS::new("https://example.example")
                .AllowCredentials()
                .AllowHeaders(["Content-Type", "X-Custom"]),
            "/".GET(|| async {"Hello!"})
        ).test(); {
            let req = TestRequest::GET("/");
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
        } {
            let req = TestRequest::OPTIONS("/");
            let res = t.oneshot(req).await;

            assert_eq!(res.status().code(), 200);
            assert_eq!(res.text(), None);

            assert_eq!(res.header("Access-Control-Allow-Origin"), Some("https://example.example"));
            assert_eq!(res.header("Access-Control-Allow-Credentials"), Some("true"));
            assert_eq!(res.header("Access-Control-Expose-Headers"), None);
            assert_eq!(res.header("Access-Control-Max-Age"), None);
            assert_eq!(res.header("Access-Control-Allow-Methods"), Some("GET, PUT, POST, PATCH, DELETE, OPTIONS, HEAD"));
            assert_eq!(res.header("Access-Control-Allow-Headers"), Some("Content-Type, X-Custom"));
            assert_eq!(res.header("Vary"), Some("Access-Control-Request-Headers"));
        }

        let t = Ohkami::with(
            CORS::new("*")
                .AllowHeaders(["Content-Type", "X-Custom"])
                .MaxAge(1024),
            "/".GET(|| async {"Hello!"})
        ).test(); {
            let req = TestRequest::OPTIONS("/");
            let res = t.oneshot(req).await;

            assert_eq!(res.status().code(), 200);
            assert_eq!(res.text(), None);

            assert_eq!(res.header("Access-Control-Allow-Origin"), Some("*"));
            assert_eq!(res.header("Access-Control-Allow-Credentials"), None);
            assert_eq!(res.header("Access-Control-Expose-Headers"), None);
            assert_eq!(res.header("Access-Control-Max-Age"), Some("1024"));
            assert_eq!(res.header("Access-Control-Allow-Methods"), Some("GET, PUT, POST, PATCH, DELETE, OPTIONS, HEAD"));
            assert_eq!(res.header("Access-Control-Allow-Headers"), Some("Content-Type, X-Custom"));
            assert_eq!(res.header("Vary"), Some("Origin, Access-Control-Request-Headers"));
        }
    }
}
