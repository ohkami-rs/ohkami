use crate::{Request, FromRequest};

#[cfg(not(feature="openapi"))]
mod bound {
    pub trait FromHeaderBound: Sized {}
    impl<T: Sized> FromHeaderBound for T {}
}
#[cfg(feature="openapi")]
mod bound {
    pub trait FromHeaderBound: Sized + crate::openapi::Schema {}
    impl<T: Sized + crate::openapi::Schema> FromHeaderBound for T {}
}

/// Parsed from raw header value ( `&'req str` )
pub trait FromHeader<'req>: bound::FromHeaderBound {
    /// If this extraction never fails, `std::convert::Infallible` is recomended.
    type Error: crate::IntoResponse;

    /// Parsing logic
    fn from_header(raw: &'req str) -> Result<Self, Self::Error>;
}
const _: () = {
    impl<'req> FromHeader<'req> for &'req str {
        type Error = std::convert::Infallible;
        fn from_header(raw: &'req str) -> Result<Self, Self::Error> {
            Ok(raw)
        }
    }

    impl<'req> FromHeader<'req> for String {
        type Error = std::convert::Infallible;
        fn from_header(raw: &'req str) -> Result<Self, Self::Error> {
            Ok(raw.into())
        }
    }
};

macro_rules! typed_header {
    ($( $Name:ident : $key:literal ),* $(,)?) => {$(
        /// Extract the request header value as an type implementing
        /// [`FromHeader<'_>`](crate::typed::header::FromHeader) trait
        /// ( By default, `&str` and `String` implement it ) .
        /// 
        /// ## Note
        /// 
        /// ## Example
        /// 
        /// here extracting `Authorization` header value as `&str`.
        /// 
        /// ```no_run
        /// use ohkami::prelude::*;
        /// use ohkami::typed::header;
        /// 
        /// async fn private_handler(
        ///     header::Authorization(a): header::Authorization<&str>,
        ///     r: Option<header::Referer<&str>>,
        /// ) -> String {
        ///     println!("Referer: `{r:?}`");
        ///     let token = a.strip_prefix("Bearer ")
        ///        .expect("expected `Bearer <token>`");
        ///     format!("got Authorization: `{token}`")
        /// }
        /// 
        /// #[tokio::main]
        /// async fn main() {
        ///     Ohkami::new((
        ///         "/private".GET(private_handler),
        ///     )).howl("localhost:5050").await
        /// }
        /// ```
        pub struct $Name<Value>(pub Value);

        impl<'req, Value: FromHeader<'req>> FromRequest<'req> for $Name<Value> {
            type Error = <Value as FromHeader<'req>>::Error;

            fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
                let raw = req.headers.$Name()?;
                Some(Value::from_header(raw).map(Self))
            }

            #[cfg(feature="openapi")]
            fn openapi_inbound() -> crate::openapi::Inbound {
                crate::openapi::Inbound::Param(crate::openapi::Parameter::in_header(
                    $key, <Value as crate::openapi::Schema>::schema()
                ))
            }
        }

        impl<'req, Value: FromHeader<'req> + std::fmt::Debug> std::fmt::Debug for $Name<Value> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(stringify!($Name))?;
                f.write_str("(")?;
                write!(f, "{:?}", self.0)?;
                f.write_str(")")?;
                Ok(())
            }
        }
    )*};
}
typed_header! {
    Accept:                      "Accept",
    AcceptEncoding:              "Accept-Encoding",
    AcceptLanguage:              "Accept-Language",
    AccessControlRequestHeaders: "Access-Control-Request-Headers",
    AccessControlRequestMethod:  "Access-Control-Request-Method",
    Authorization:               "Authorization",
    CacheControl:                "Cache-Control",
    Connection:                  "Connection",
    ContentDisposition:          "Content-Disposition",
    ContentEncoding:             "Content-Encoding",
    ContentLanguage:             "Content-Language",
    ContentLength:               "Content-Length",
    ContentLocation:             "Content-Location",
    ContentType:                 "Content-Type",
    /* Cookie:                      "Cookie", // specialize */
    Date:                        "Date",
    Expect:                      "Expect",
    Forwarded:                   "Forwarded",
    From:                        "From",
    Host:                        "Host",
    IfMatch:                     "If-Match",
    IfModifiedSince:             "If-Modified-Since",
    IfNoneMatch:                 "If-None-Match",
    IfRange:                     "If-Range",
    IfUnmodifiedSince:           "If-Unmodified-Since",
    Link:                        "Link",
    MaxForwards:                 "Max-Forwards",
    Origin:                      "Origin",
    ProxyAuthorization:          "Proxy-Authorization",
    Range:                       "Range",
    Referer:                     "Referer",
    SecFetchDest:                "Sec-Fetch-Dest",
    SecFetchMode:                "Sec-Fetch-Mode",
    SecFetchSite:                "Sec-Fetch-Site",
    SecFetchUser:                "Sec-Fetch-User",
    SecWebSocketExtensions:      "Sec-WebSocket-Extensions",
    SecWebSocketKey:             "Sec-WebSocket-Key",
    SecWebSocketProtocol:        "Sec-WebSocket-Protocol",
    SecWebSocketVersion:         "Sec-WebSocket-Version",
    TE:                          "TE",
    Trailer:                     "Trailer",
    TransferEncoding:            "Transfer-Encoding",
    UserAgent:                   "User-Agent",
    Upgrade:                     "Upgrade",
    UpgradeInsecureRequests:     "Upgrade-Insecure-Requests",
    Via:                         "Via",
}

/// Extract `Cookie` header value and parse to a type implementing `Deserialize<'_>`.
/// 
/// ## Note
/// 
/// In `openapi` feature activated, the type is also required to impl `openapi::Schema`.
/// 
/// ## Example
/// 
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::typed::header;
/// 
/// #[derive(Deserialize)]
/// struct CookieSchema<'req> {
///     session_id: &'req str,
///     metadata: Option<&'req str>,
/// }
/// 
/// /// expecting request headers contains something like:
/// /// 
/// /// - `Cookie: session_id=ABCDEFG`
/// /// - `Cookie: metadata="Hello, 世界!"; session_id=XYZ123`
/// async fn private_handler(
///     header::Cookie(c): header::Cookie<CookieSchema<'_>>,
/// ) {
///     println!("session_id: `{}`", c.session_id);
///     println!("metadata: `{:?}`", c.metadata);
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::new((
///         "/private".GET(private_handler),
///     )).howl("localhost:5050").await
/// }
/// ```
pub struct Cookie<Fields>(pub Fields);

impl<'req, Fields: crate::format::bound::Incoming<'req>> FromRequest<'req> for Cookie<Fields> {
    type Error = crate::typed::status::Unauthorized<String>;

    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        req.headers.Cookie().map(|raw| ohkami_lib::serde_cookie::from_str::<Fields>(raw)
            .map(Cookie)
            .map_err(|e| crate::typed::status::Unauthorized(format!(
                "missing or invalid Cookie: {e}"
            )))
        )
    }

    #[cfg(feature="openapi")]
    fn openapi_inbound() -> crate::openapi::Inbound {
        let Some(schema) = Fields::schema().into().into_inline() else {
            return crate::openapi::Inbound::None
        };
        crate::openapi::Inbound::Params(
            schema.into_properties().into_iter().map(|(name, schema, required)|
                if required {
                    crate::openapi::Parameter::in_cookie(name, schema)
                } else {
                    crate::openapi::Parameter::maybe_in_cookie(name, schema)
                }
            ).collect()
        )
    }
}
