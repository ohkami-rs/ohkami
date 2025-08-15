use crate::{Request, Response, IntoResponse, Fang, FangProc};
use std::sync::Arc;

/// # Built-in CSRF protection fang.
/// 
/// The implementation is based on  the way of Go 1.25 net/http's `CrossOriginProtection`:
/// 
/// - doc: https://go.dev/doc/go1.25#nethttppkgnethttp
/// - code: https://cs.opensource.google/go/go/+/refs/tags/go1.25.0:src/net/http/csrf.go
/// 
/// providing a token-less CSRF protection mechanism, with support for byppassing trusted origins.
/// 
/// ## Usage
/// 
/// ### Single Server Service
/// 
/// Just `Csrf::new()` and add it to your Ohkami app.
/// 
/// ```no_run
/// use ohkami::{Ohkami, Route, fang::Csrf};
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         Csrf::new(),
///         "/".GET(|| async {"Hello, CSRF!"}),
///     ))
/// }
/// ```
/// 
/// ### Multi Server Service
/// 
/// If you have multiple servers, you can use `Csrf::with_trusted_origins`
/// to specify trusted origins.
/// 
/// ```no_run
/// use ohkami::{Ohkami, Route, fang::Csrf};
///
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         Csrf::with_trusted_origins([
///             "https://example.com",
///             "https://example.org",
///         ]),
///         "/".GET(|| async {"Hello, CSRF!"}),
///     ))
/// }
/// ```
#[derive(Clone)]
pub struct Csrf {
    trusted_origins: Arc<Vec<&'static str>>,
}

impl Csrf {
    pub fn new() -> Self {
        Csrf {
            trusted_origins: Arc::new(vec![]),
        }
    }
    
    pub fn with_trusted_origins(trusted_origins: impl IntoIterator<Item = &'static str>) -> Self {
        let trusted_origins = trusted_origins.into_iter().collect::<Vec<_>>();
        
        for origin in &trusted_origins {
            let Some((scheme, rest)) = origin.split_once("://") else {
                panic!("invalid origin `{origin}`: scheme is required")
            };
            if !matches!(scheme, "http" | "https") {
                panic!("invalid origin `{origin}`: scheme must be 'http' or 'https'");
            }
            if rest.contains(['/', '?', '#']) {
                panic!("invalid origin `{origin}`: path, query and fragment are not allowed");
            }
            if rest.is_empty() || !rest.starts_with(|x: char| x.is_ascii_alphanumeric()) {
                panic!("invalid origin `{origin}`: host is required");
            }
        }
        
        Csrf { trusted_origins: Arc::new(trusted_origins) }
    }
}

pub enum CsrfError {
    InvalidSecFetchSite,
    OriginNotMatchHost,
    NoHostHeader,
}
impl IntoResponse for CsrfError {
    fn into_response(self) -> Response {
        match self {
            CsrfError::InvalidSecFetchSite => Response::Forbidden()
                .with_text("cross-origin request detected from Sec-Fetch-Site header"),
            CsrfError::OriginNotMatchHost => Response::Forbidden()
                .with_text("cross-origin request detected, and/or browser is out of date: Sec-Fetch-Site is missing, and Origin does not match Host"),
            CsrfError::NoHostHeader => Response::BadRequest(),
        }
    }
}

impl Csrf {
    pub fn verify(&self, req: &Request) -> Result<(), CsrfError> {
        let is_trusted = || req.headers.origin().is_some_and(|it| self.trusted_origins.contains(&it));
        
        if req.method.is_safe() {
            Ok(())
        } else if let Some(sec_fetch_site) = req.headers.sec_fetch_site() {
            match sec_fetch_site {
                "same-origin" | "none" => Ok(()),
                _ => is_trusted().then_some(()).ok_or(CsrfError::InvalidSecFetchSite),
            }
        } else {
            match (req.headers.origin(), req.headers.host()) {
                (None, _) => Ok(()), // No Origin header, so we assume it's same-origin or not a browser request.
                (_, None) => Err(CsrfError::NoHostHeader),
                (Some(origin), Some(host)) if matches!(
                    origin.strip_suffix(host),
                    Some("http://" | "https://")
                ) => Ok(()),
                _ => is_trusted().then_some(()).ok_or(CsrfError::OriginNotMatchHost),
            }
        }
    }
}

const _: () = {
    pub struct CsrfProc<I: FangProc> {
        csrf: Csrf,
        inner: I,
    }
    
    impl<I: FangProc> Fang<I> for Csrf {
        type Proc = CsrfProc<I>;

        fn chain(&self, inner: I) -> Self::Proc {
            CsrfProc { csrf: self.clone(), inner }
        }
    }
    
    impl<I: FangProc> FangProc for CsrfProc<I> {
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            match self.csrf.verify(req) {
                Ok(()) => self.inner.bite(req).await,
                Err(e) => e.into_response(),
            }
        }
    }
};
