use crate::{Fang, FangProc, IntoResponse, Request, Response};
use std::sync::Arc;

/// # Built-in CSRF protection fang
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
///     )).run("0.0.0.0:3000").await
/// }
/// ```
///
/// ### Multi Server Service
///
/// If you have multiple servers, you can use `Csrf::with_trusted_origins`
/// to specify trusted origins.
///
/// **NOTE**: wildcards (like `https://*.a.domain`) are not supported in trusted origins.
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
///     )).run("0.0.0.0:5000").await
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Csrf {
    trusted_origins: Arc<Vec<&'static str>>,
}

impl Default for Csrf {
    fn default() -> Self {
        Self::new()
    }
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
            let Some(("http" | "https", rest)) = origin.split_once("://") else {
                panic!(
                    "[Csrf::with_trusted_origins] invalid origin: 'http' or 'https' scheme is required"
                )
            };
            let (host, port) = rest
                .split_once(':')
                .map_or((rest, None), |(h, p)| (h, Some(p)));
            if port.is_some_and(|p| !p.chars().all(|c| c.is_ascii_digit())) {
                panic!("[Csrf::with_trusted_origins] invalid origin: port must be a number");
            }
            if !host.starts_with(|c: char| c.is_ascii_alphabetic()) {
                panic!(
                    "[Csrf::with_trusted_origins] invalid origin: host must start with an alphabetic character"
                );
            }
            if !host.split('.').all(|part| {
                !part.is_empty()
                    && part
                        .chars()
                        .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_'))
            }) {
                if host.contains(['/', '?', '#']) {
                    // helpful error message for common mistake
                    panic!(
                        "[Csrf::with_trusted_origins] invalid origin: path, query and fragment are not allowed"
                    );
                } else {
                    panic!("[Csrf::with_trusted_origins] invalid origin: invalid host");
                }
            }
        }

        Csrf {
            trusted_origins: Arc::new(trusted_origins),
        }
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
        let is_trusted = || {
            req.headers
                .origin()
                .is_some_and(|it| self.trusted_origins.contains(&it))
        };

        if req.method.is_safe() {
            Ok(())
        } else if let Some(sec_fetch_site) = req.headers.sec_fetch_site() {
            match sec_fetch_site {
                "same-origin" | "none" => Ok(()),
                _ => is_trusted()
                    .then_some(())
                    .ok_or(CsrfError::InvalidSecFetchSite),
            }
        } else {
            match (req.headers.origin(), req.headers.host()) {
                (None, _) => Ok(()), // No Origin header, so we assume it's same-origin or not a browser request.
                (_, None) => Err(CsrfError::NoHostHeader),
                (Some(origin), Some(host))
                    if matches!(origin.strip_suffix(host), Some("http://" | "https://")) =>
                {
                    Ok(())
                }
                _ => is_trusted()
                    .then_some(())
                    .ok_or(CsrfError::OriginNotMatchHost),
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
            CsrfProc {
                csrf: self.clone(),
                inner,
            }
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

#[cfg(test)]
#[cfg(feature = "__rt_native__")]
mod tests {
    //! based on https://cs.opensource.google/go/go/+/refs/tags/go1.25.0:src/net/http/csrf_test.go

    use super::*;
    use crate::testing::*;
    use crate::{Ohkami, Route};

    macro_rules! x {
        ($method:ident) => {
            TestRequest::$method("/").header("host", "example.com")
        };
    }

    #[test]
    fn test_sec_fetch_site() {
        let t = Ohkami::new((
            Csrf::new(),
            "/".GET(async || ()).PUT(async || ()).POST(async || ()),
        ))
        .test();

        crate::__rt__::testing::block_on(async {
            for (req, expected) in [
                (x!(POST).header("sec-fetch-site", "same-origin"), Status::OK),
                (x!(POST).header("sec-fetch-site", "none"), Status::OK),
                (
                    x!(POST).header("sec-fetch-site", "cross-site"),
                    Status::Forbidden,
                ),
                (
                    x!(POST).header("sec-fetch-site", "same-site"),
                    Status::Forbidden,
                ),
                (x!(POST), Status::OK),
                (x!(POST).header("origin", "https://example.com"), Status::OK),
                (
                    x!(POST).header("origin", "https://attacker.example"),
                    Status::Forbidden,
                ),
                (x!(POST).header("origin", "null"), Status::Forbidden),
                (x!(GET).header("sec-fetch-site", "cross-site"), Status::OK),
                (x!(HEAD).header("sec-fetch-site", "cross-site"), Status::OK),
                (
                    x!(OPTIONS).header("sec-fetch-site", "cross-site"),
                    Status::NotFound,
                ), // see `fang::handler::Handler::default_options_with`
                (
                    x!(PUT).header("sec-fetch-site", "cross-site"),
                    Status::Forbidden,
                ),
            ] {
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), expected);
            }
        });
    }

    #[test]
    fn test_trusted_origins() {
        let t = Ohkami::new((
            Csrf::with_trusted_origins(["https://trusted.example"]),
            "/".POST(async || ()),
        ))
        .test();

        crate::__rt__::testing::block_on(async {
            for (req, expected) in [
                (
                    x!(POST).header("origin", "https://trusted.example"),
                    Status::OK,
                ),
                (
                    x!(POST)
                        .header("origin", "https://trusted.example")
                        .header("sec-fetch-site", "cross-site"),
                    Status::OK,
                ),
                (
                    x!(POST).header("origin", "https://attacker.example"),
                    Status::Forbidden,
                ),
                (
                    x!(POST)
                        .header("origin", "https://attacker.example")
                        .header("sec-fetch-site", "cross-site"),
                    Status::Forbidden,
                ),
            ] {
                let res = t.oneshot(req).await;
                assert_eq!(res.status(), expected);
            }
        });
    }

    #[test]
    fn test_invalid_trusted_origins() {
        for (trusted_origin, should_judged_as_invalid) in [
            ("https://example.com", false),
            ("https://example.com:8080", false),
            ("http://example.com", false),
            ("example.com", true),                  // missing scheme
            ("https://", true),                     // missing host
            ("https://example.com/", true),         // path is not allowed
            ("https://example.com/path", true),     // path is not allowed
            ("https://example.com?query=1", true),  // query is not allowed
            ("https://example.com#fragment", true), // fragment is not allowed
            ("https://ex ample.com", true),         // invalid host
            ("", true),                             // empty string
            ("null", true),                         // missing scheme
            ("https://example.com:port", true),     // invalid port
        ] {
            let is_judged_as_invalid = std::panic::catch_unwind(|| {
                let _ = Csrf::with_trusted_origins([trusted_origin]);
            })
            .is_err();
            assert_eq!(
                is_judged_as_invalid, should_judged_as_invalid,
                "unexpected result for trusted origin `{trusted_origin}`"
            );
        }
    }
}
