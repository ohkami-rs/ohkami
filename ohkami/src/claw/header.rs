use crate::{FromRequest, Request};

#[cfg(not(feature = "openapi"))]
mod bound {
    pub trait FromHeaderBound: Sized {}
    impl<T: Sized> FromHeaderBound for T {}
}
#[cfg(feature = "openapi")]
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
    ($( $method:ident($Name:ident) : $key:literal ),* $(,)?) => {$(
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
        /// use ohkami::{Ohkami, Route};
        /// use ohkami::claw::header;
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
        ///     )).run("localhost:5050").await
        /// }
        /// ```
        pub struct $Name<Value>(pub Value);

        impl<'req, Value: FromHeader<'req>> FromRequest<'req> for $Name<Value> {
            type Error = <Value as FromHeader<'req>>::Error;

            fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
                let raw = req.headers.$method()?;
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
    accept(Accept): "Accept",
    accept_encoding(AcceptEncoding): "Accept-Encoding",
    accept_language(AcceptLanguage): "Accept-Language",
    access_control_request_headers(AccessControlRequestHeaders): "Access-Control-Request-Headers",
    access_control_request_method(AccessControlRequestMethod): "Access-Control-Request-Method",
    authorization(Authorization): "Authorization",
    cache_control(CacheControl): "Cache-Control",
    connection(Connection): "Connection",
    content_disposition(ContentDisposition): "Content-Disposition",
    content_encoding(ContentEncoding): "Content-Encoding",
    content_laguage(ContentLanguage): "Content-Language",
    content_length(ContentLength): "Content-Length",
    content_location(ContentLocation): "Content-Location",
    content_type(ContentType): "Content-Type",
    /* cookie(Cookie): "Cookie", : specialize later... */
    date(Date): "Date",
    expect(Expect): "Expect",
    forwarded(Forwarded): "Forwarded",
    from(From): "From",
    host(Host): "Host",
    if_match(IfMatch): "If-Match",
    if_modified_since(IfModifiedSince): "If-Modified-Since",
    if_none_match(IfNoneMatch): "If-None-Match",
    if_range(IfRange): "If-Range",
    if_unmodified_since(IfUnmodifiedSince): "If-Unmodified-Since",
    link(Link): "Link",
    max_forwards(MaxForwards): "Max-Forwards",
    origin(Origin): "Origin",
    proxy_authorization(ProxyAuthorization): "Proxy-Authorization",
    range(Range): "Range",
    referer(Referer): "Referer",
    sec_fetch_dest(SecFetchDest): "Sec-Fetch-Dest",
    sec_fetch_mode(SecFetchMode): "Sec-Fetch-Mode",
    sec_fetch_site(SecFetchSite): "Sec-Fetch-Site",
    sec_fetch_user(SecFetchUser): "Sec-Fetch-User",
    sec_websocket_extensions(SecWebSocketExtensions): "Sec-WebSocket-Extensions",
    sec_websocket_key(SecWebSocketKey): "Sec-WebSocket-Key",
    sec_websocket_protocol(SecWebSocketProtocol): "Sec-WebSocket-Protocol",
    sec_websocket_version(SecWebSocketVersion): "Sec-WebSocket-Version",
    te(TE): "TE",
    trailer(Trailer): "Trailer",
    transfer_encoding(TransferEncoding): "Transfer-Encoding",
    user_agent(UserAgent): "User-Agent",
    upgrade(Upgrade): "Upgrade",
    upgrade_insecure_requests(UpgradeInsecureRequests): "Upgrade-Insecure-Requests",
    via(Via): "Via",
}

/// Extract `Cookie` header value and parse to a type implementing `Deserialize<'_>`.
///
/// ## Note
///
/// When `openapi` feature activated, the type is also required to impl `openapi::Schema`.
///
/// ## Example
///
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::claw::header;
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
///     )).run("localhost:5050").await
/// }
/// ```
pub struct Cookie<Fields>(pub Fields);

impl<'req, Fields: super::bound::Incoming<'req>> FromRequest<'req> for Cookie<Fields> {
    type Error = crate::claw::status::Unauthorized<&'static str>;

    fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
        req.headers.cookie().map(|raw| {
            ohkami_lib::serde_cookie::from_str::<Fields>(raw)
                .map(Cookie)
                .map_err(|_e| {
                    #[cfg(debug_assertions)]
                    {
                        crate::WARNING!(
                            "{_e:?}: failed to parse Cookie header as `{}`: `{raw}`",
                            std::any::type_name::<Fields>()
                        );
                    }
                    crate::claw::status::Unauthorized("missing or invalid Cookie")
                })
        })
    }

    #[cfg(feature = "openapi")]
    fn openapi_inbound() -> crate::openapi::Inbound {
        let Some(schema) = Fields::schema().into().into_inline() else {
            return crate::openapi::Inbound::None;
        };
        crate::openapi::Inbound::Params(
            schema
                .into_properties()
                .into_iter()
                .map(|(name, schema, required)| {
                    if required {
                        crate::openapi::Parameter::in_cookie(name, schema)
                    } else {
                        crate::openapi::Parameter::in_cookie_optional(name, schema)
                    }
                })
                .collect(),
        )
    }
}
