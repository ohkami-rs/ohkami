macro_rules! ClientHeader {
    ($N:literal; $( $konst:ident: $name_bytes:literal $(| $other_case:literal)*, )*) => {
        pub(crate) const N_CLIENT_HEADERS: usize = $N;
        pub(crate) const CLIENT_HEADERS: [ClientHeader; N_CLIENT_HEADERS] = [ $( ClientHeader::$konst ),* ];

        #[derive(Debug, PartialEq)]
        pub enum ClientHeader {
            $( $konst, )*
        }

        impl ClientHeader {
            #[inline] pub fn as_str(&self) -> &'static str {
                match self {
                    $(
                        Self::$konst => unsafe {std::str::from_utf8_unchecked($name_bytes)},
                    )*
                }
            }
            #[inline] pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
                match bytes {
                    $(
                        $name_bytes $(| $other_case)* => Some(Self::$konst),
                    )*
                    _ => None
                }
            }
        }

        #[cfg(test)] #[test] fn client_header_name_cases() {
            $(
                $(
                    assert_eq!($name_bytes.to_ascii_lowercase(), $other_case);
                )*
            )*
        }
    };
} ClientHeader! {41;
    Accept:                  b"Accept" | b"accept",
    AcceptEncoding:          b"Accept-Encoding" | b"accept-encoding",
    AcceptLanguage:          b"Accept-Language" | b"accept-language",
    Authorization:           b"Authorization" | b"authorization",
    CacheControl:            b"Cache-Control" | b"cache-control",
    Connection:              b"Connection" | b"connection",
    ContentDisposition:      b"Content-Disposition" | b"content-disposition",
    ContentEncoding:         b"Content-Encoding" | b"content-encoding",
    ContentLanguage:         b"Content-Language" | b"content-language",
    ContentLength:           b"Content-Length" | b"content-length",
    ContentLocation:         b"Content-Location" | b"content-location",
    ContentType:             b"Content-Type" | b"content-type",
    Cookie:                  b"Cookie" | b"cookie",
    Date:                    b"Date" | b"date",
    Expect:                  b"Expect" | b"expect",
    Forwarded:               b"Forwarded" | b"forwarded",
    From:                    b"From" | b"from",
    Host:                    b"Host" | b"host",
    IfMatch:                 b"If-Match" | b"if-match",
    IfModifiedSince:         b"If-Modified-Since" | b"if-modified-since",
    IfNoneMatch:             b"If-None-Match" | b"if-none-match",
    IfRange:                 b"If-Range" | b"if-range",
    IfUnmodifiedSince:       b"If-Unmodified-Since" | b"if-unmodified-since",
    Link:                    b"Link" | b"link",
    MaxForwards:             b"Max-Forwards" | b"max-forwards",
    Origin:                  b"Origin" | b"origin",
    ProxyAuthorization:      b"Proxy-Authorization" | b"proxy-authorization",
    Range:                   b"Range" | b"range",
    Referer:                 b"Referer" | b"referer",
    SecWebSocketExtensions:  b"Sec-WebSocket-Extensions" | b"sec-websocket-extensions",
    SecWebSocketKey:         b"Sec-WebSocket-Key" | b"sec-websocket-key",
    SecWebSocketProtocol:    b"Sec-WebSocket-Protocol" | b"sec-websocket-protocol",
    SecWebSocketVersion:     b"Sec-WebSocket-Version" | b"sec-websocket-version",
    TE:                      b"TE" | b"te",
    Trailer:                 b"Trailer" | b"trailer",
    TransferEncoding:        b"Transfer-Encoding" | b"transfer-encoding",
    UserAgent:               b"User-Agent" | b"user-agent",
    Upgrade:                 b"Upgrade" | b"upgrade",
    UpgradeInsecureRequests: b"Upgrade-Insecure-Requests" | b"upgrade-insecure-requests",
    Via:                     b"Via" | b"via",
    XRequestID:              b"X-Request-ID" | b"X-Request-Id" | b"x-request-id",
}

macro_rules! ServerHeader {
    ($N:literal; $( $konst:ident: $name_bytes:literal, )*) => {
        pub(crate) const N_SERVER_HEADERS: usize = $N;
        pub(crate) const SERVER_HEADERS: [ServerHeader; N_SERVER_HEADERS] = [ $( ServerHeader::$konst ),* ];

        #[derive(Debug, PartialEq, Clone, Copy)]
        pub enum ServerHeader {
            $( $konst, )*
        }

        impl ServerHeader {
            #[inline] pub fn as_bytes(&self) -> &'static [u8] {
                match self {
                    $(
                        Self::$konst => $name_bytes,
                    )*
                }
            }
            #[inline] pub fn as_str(&self) -> &'static str {
                unsafe {std::str::from_utf8_unchecked(self.as_bytes())}
            }
        }
    };
} ServerHeader! {47;
    AcceptRanges:                    b"Accept-Ranges",
    AccessControlAllowCredentials:   b"Access-Control-Allow-Credentials",
    AccessControlAllowHeaders:       b"Access-Control-Allow-Headers",
    AccessControlAllowMethods:       b"Access-Control-Allow-Methods",
    AccessControlAllowOrigin:        b"Access-Control-Allow-Origin",
    AccessControlExposeHeaders:      b"Access-Control-Expose-Headers",
    AccessControlMaxAge:             b"Access-Control-Max-Age",
    AccessControlRequestHeaders:     b"Access-Control-Request-Headers",
    AccessControlRequestMethod:      b"Access-Control-Request-Method",
    Age:                             b"Age",
    Allow:                           b"Allow",
    AltSrv:                          b"Alt-Srv",
    CacheControl:                    b"Cache-Control",
    CacheStatus:                     b"Cache-Status",
    CDNCacheControl:                 b"CDN-Cache-Control",
    Connection:                      b"Connection",
    ContentDisposition:              b"Content-Disposition",
    ContentEncoding:                 b"Content-Ecoding",
    ContentLanguage:                 b"Content-Language",
    ContentLength:                   b"Content-Length",
    ContentLocation:                 b"Content-Location",
    ContentRange:                    b"Content-Range",
    ContentSecurityPolicy:           b"Content-Security-Policy",
    ContentSecurityPolicyReportOnly: b"Content-Security-Policy-Report-Only",
    ContentType:                     b"Content-Type",
    Date:                            b"Date",
    ETag:                            b"ETag",
    Expires:                         b"Expires",
    Link:                            b"Link",
    Location:                        b"Location",
    ProxyAuthenticate:               b"Proxy-Authenticate",
    ReferrerPolicy:                  b"Referrer-Policy",
    Refresh:                         b"Refresh",
    RetryAfter:                      b"Retry-After",
    SecWebSocketAccept:              b"Sec-WebSocket-Accept",
    SecWebSocketProtocol:            b"Sec-WebSocket-Protocol",
    SecWebSocketVersion:             b"Sec-WebSocket-Version",
    Server:                          b"Server",
    SetCookie:                       b"SetCookie",
    StrictTransportSecurity:         b"Strict-Transport-Security",
    Trailer:                         b"Trailer",
    TransferEncoding:                b"Transfer-Encoding",
    Upgrade:                         b"Upgrade",
    Vary:                            b"Vary",
    Via:                             b"Via",
    XContentTypeOptions:             b"X-Content-Type-Options",
    XFrameOptions:                   b"X-Frame-Options",
}
