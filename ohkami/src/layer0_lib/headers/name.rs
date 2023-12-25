macro_rules! HeaderName {
    ($( $konst:ident: $name_bytes:literal, )*) => {
        /// In the current version, ohkami doesn't support custom HTTP headers
        /// both in request processing or response generation.
        /// 
        /// In future, ohkami will introduce a feature like `"custom_headers"` or something,
        /// with that you can do in exchange for a slight decrease in performance.
        #[derive(Debug, PartialEq, Eq, Hash)]
        pub enum HeaderName {
            $( $konst, )*
        }
        impl HeaderName {
            #[inline] pub fn as_str(&self) -> &'static str {
                match self {
                    $(
                        Self::$konst => unsafe {std::str::from_utf8_unchecked($name_bytes)},
                    )*
                }
            }
            const fn from_bytes(bytes: &[u8]) -> Option<Self> {
                match bytes {
                    $(
                        $name_bytes => Some(Self::$konst),
                    )*
                    _ => None
                }
            }
        }
    };
} HeaderName! {
    AccessControlAllowCredentials: b"Access-Control-Allow-Credentials",
    AccessControlAllowHeaders:     b"Access-Control-Allow-Headers",
    AccessControlAllowMethods:     b"Access-Control-Allow-Methods",
    AccessControlAllowOrigin:      b"Access-Control-Allow-Origin",
    AccessControlExposeHeaders:    b"Access-Control-Expose-Headers",
    AccessControlMaxAge:           b"Access-Control-Max-Age",
    AccessControlRequestHeaders:   b"Access-Control-Request-Headers",
    AccessControlRequestMethod:    b"Access-Control-Request-Method",

    Accept: b"Accept",
    AcceptEncoding: b"Accept-Encoding",
    AcceptLanguage: b"Accept-Language",
    AcceptRanges: b"Accept-Ranges",
    Age: b"Age",
    Allow: b"Allow",
    AltSrv: b"Alt-Srv",
    Authorization: b"Authorization",
    CacheControl: b"Cache-Control",
    CacheStatus: b"CacheStatus",
    CDNCacheControl: b"CDN-Cache-Control",
    Connection: b"Connection",
    ContentDisposition: b"Content-Disposition",
    ContentEncoding: b"Content-Ecoding",
    ContentLanguage: b"Content-Language",
    ContentLength: b"Content-Length",
    ContentLocation: b"Content-Location",
    ContentRange: b"Content-Range",
    ContentSecurityPolicyReportOnly: b"Content-Security-Policy-Report-Only",
    ContentType: b"Content-Type",
    Cookie: b"Cookie",
    Date: b"Date",
    ETag: b"ETag",
    Expect: b"Expect",
    Expires: b"Expires",
    Forwarded: b"Forwarded",
    From: b"From",
    Host: b"Host",
    IfMatch: b"If-Match",
    IfModifiedSince: b"If-Modified-Since",
    IfNoneMatch: b"If-None-Match",
    IfRange: b"If-Range",
    IfUnmodifiedSince: b"If-Unmodified-Since",
    Link: b"Link",
    Location: b"Location",
    MaxForwards: b"Max-Forwards",
    Origin: b"Origin",
    ProxyAuthenticate: b"Proxy-Authenticate",
    ProxyAuthorization: b"Proxy-Authorization",
    Range: b"Range",
    Referer: b"Referer",
    ReferrerPolicy: b"Referrer-Policy",
    Refresh: b"Refresh",
    RetryAfter: b"Retry-After",
    SecWebSocketAccept: b"Sec-WebSocket-Accept",
    SecWebSocketExtensions: b"Sec-WebSocket-Extensions",
    SecWebSocketKey: b"Sec-WebSocket-Key",
    SecWebSocketProtocol: b"Sec-WebSocket-Protocol",
    SecWebSocketVersion: b"Sec-WebSocket-Version",
    Server: b"Server",
    SetCookie: b"SetCookie",
    StrictTransportSecurity: b"Strict-Transport-Security",
    TE: b"TE",
    Trailer: b"Trailer",
    TransferEncoding: b"Transfer-Encoding",
    UserAgent: b"User-Agent",
    Upgrade: b"Upgrade",
    UpgradeInsecureRequests: b"Upgrade-Insecure-Requests",
    Vary: b"Vary",
    Via: b"Via",
    XContentTypeOptions: b"X-Content-Type-Options",
    XFrameOptions: b"X-Frame-Options",
}

pub trait IntoHeaderName {
    fn into_header_name(self) -> Option<HeaderName>;
} impl<'s, C: Into<std::borrow::Cow<'s, str>>> IntoHeaderName for C {
    #[inline] fn into_header_name(self) -> Option<HeaderName> {
        HeaderName::from_bytes(self.into().as_bytes())
    }
}


/*

[from client]

    Accept: b"Accept",
    AcceptEncoding: b"Accept-Encoding",
    AcceptLanguage: b"Accept-Language",
    Authorization: b"Authorization",
    CacheControl: b"Cache-Control",
    Connection: b"Connection",
    ContentDisposition: b"Content-Disposition",
    ContentEncoding: b"Content-Ecoding",
    ContentLanguage: b"Content-Language",
    ContentLength: b"Content-Length",
    ContentLocation: b"Content-Location",
    ContentType: b"Content-Type",
    Cookie: b"Cookie",
    Date: b"Date",
    Expect: b"Expect",
    Forwarded: b"Forwarded",
    From: b"From",
    Host: b"Host",
    IfMatch: b"If-Match",
    IfModifiedSince: b"If-Modified-Since",
    IfNoneMatch: b"If-None-Match",
    IfRange: b"If-Range",
    IfUnmodifiedSince: b"If-Unmodified-Since",
    Link: b"Link",
    MaxForwards: b"Max-Forwards",
    Origin: b"Origin",
    ProxyAuthorization: b"Proxy-Authorization",
    Range: b"Range",
    Referer: b"Referer",
    SecWebSocketExtensions: b"Sec-WebSocket-Extensions",
    SecWebSocketKey: b"Sec-WebSocket-Key",
    SecWebSocketProtocol: b"Sec-WebSocket-Protocol",
    SecWebSocketVersion: b"Sec-WebSocket-Version",
    TE: b"TE",
    Trailer: b"Trailer",
    TransferEncoding: b"Transfer-Encoding",
    UserAgent: b"User-Agent",
    Upgrade: b"Upgrade",
    UpgradeInsecureRequests: b"Upgrade-Insecure-Requests",
    Via: b"Via",

---

[from server]

    AccessControlAllowCredentials: b"Access-Control-Allow-Credentials",
    AccessControlAllowHeaders:     b"Access-Control-Allow-Headers",
    AccessControlAllowMethods:     b"Access-Control-Allow-Methods",
    AccessControlAllowOrigin:      b"Access-Control-Allow-Origin",
    AccessControlExposeHeaders:    b"Access-Control-Expose-Headers",
    AccessControlMaxAge:           b"Access-Control-Max-Age",
    AccessControlRequestHeaders:   b"Access-Control-Request-Headers",
    AccessControlRequestMethod:    b"Access-Control-Request-Method",

    AcceptRanges: b"Accept-Ranges",
    Age: b"Age",
    Allow: b"Allow",
    AltSrv: b"Alt-Srv",
    CacheControl: b"CacheControl",
    CacheStatus: b"CacheStatus",
    CDNCacheControl: b"CDN-Cache-Control",
    Connection: b"Connection",
    ContentDisposition: b"Content-Disposition",
    ContentEncoding: b"Content-Ecoding",
    ContentLanguage: b"Content-Language",
    ContentLength: b"Content-Length",
    ContentLocation: b"Content-Location",
    ContentRange: b"Content-Range",
    ContentSecurityPolicy: b"Content-Security-Policy",
    ContentSecurityPolicyReportOnly: b"Content-Security-Policy-Report-Only",
    ContentType: b"Content-Type",
    Date: b"Date",
    ETag: b"ETag",
    Expires: b"Expires",
    Link: b"Link",
    Location: b"Location",
    ProxyAuthenticate: b"Proxy-Authenticate",
    ReferrerPolicy: b"Referrer-Policy",
    Refresh: b"Refresh",
    RetryAfter: b"Retry-After",
    SecWebSocketAccept: b"Sec-WebSocket-Accept",
    SecWebSocketProtocol: b"Sec-WebSocket-Protocol",
    SecWebSocketVersion: b"Sec-WebSocket-Version",
    Server: b"Server",
    SetCookie: b"SetCookie",
    StrictTransportSecurity: b"Strict-Transport-Security",
    Trailer: b"Trailer",
    TransferEncoding: b"Transfer-Encoding",
    Upgrade: b"Upgrade",
    Vary: b"Vary",
    Via: b"Via",
    XContentTypeOptions: b"X-Content-Type-Options",
    XFrameOptions: b"X-Frame-Options",

*/
