use ohkami_lib::CowSlice;
use rustc_hash::FxHasher;
use core::hash::{BuildHasherDefault, Hasher};
use core::{ops::BitXor, mem::size_of};
use std::{borrow::Cow, collections::HashMap};


pub struct HeaderMap {
    map:  HashMap<Header, Cow<'static, str>, BuildHasherDefault<HeaderHasher>>,
    size: usize
}

macro_rules! Header {
    ($( $index:literal : $name:ident = $name_pascal:literal $(| $name_lower:literal)?)*) => {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
        enum Header {
            $(
                $name = $index,
            )*
        }
        impl Header {
            #[inline(always)]
            const fn from_bytes(bytes: &[u8]) -> Self {
                match bytes {
                    $(
                        $name_pascal $(| $name_lower)? => Self::$name,
                    )*
                    _ => Self::Custom(bytes)
                }
            }
            #[inline(always)]
            const fn as_bytes(&self) -> &'static [u8] {
                match Self {
                    $(
                        Self::$name => $name_pascal,
                    )*
                    Self::Custom(bytes) => bytes
                }
            }
        }

        #[derive(Default)]
        struct HeaderHasher {
            hash: usize
        }
        impl Hasher for HeaderHasher {
            #[inline]
            fn write(&mut self, bytes: &[u8]) {
                self.hash = match bytes {
                    $(
                        $name_pascal:literal $(| $name_lower:literal)? => $index,
                    )*
                    custom => return self.write_fxhash(custom)
                }
            }
            fn finish(self) -> u64 {
                self.hash
            }
        }

        impl HeaderMap {
            pub fn set(&mut self) -> SetHeader<'_> {
                SetHeader(self)
            }
        }

        struct SetHeader<'s>(&'s mut HeaderMap);

        trait SetHeaderAction<'s> {
            fn perform(self, key: Header, set: SetHeader<'s>) -> SetHeader<'s>;
        } const _: () = {
            impl<'s> SetHeader<'s> for Option<()> {
                fn perform(self, key: Header, set: SetHeader<'s>) -> SetHeader<'s> {
                    set.0.remove(key);
                    set
                }
            }

            impl<'s> SetHeader<'s> for &'static str {
                fn perform(self, key: Header, set: SetHeader<'s>) -> SetHeader<'s> {
                    set.0.insert(key, self);
                    set
                }
            }
            impl<'s> SetHeader<'s> for &String {
                fn perform(self, key: Header, set: SetHeader<'s>) -> SetHeader<'s> {
                    set.0.insert(key, self.clone());
                    set
                }
            }
            impl<'s> SetHeader<'s> for String {
                fn perform(self, key: Header, set: SetHeader<'s>) -> SetHeader<'s> {
                    set.0.insert(key, self);
                    set
                }
            }
        };

        #[allow(non_snake_case)]
        impl<'s> SetHeader<'s> {
            $(
                pub fn $name(self, action: impl SetHeaderAction<'s>) -> Self {
                    action.perform(Self::$name, self.0)
                }
            )*
            pub fn custom(self, key: &'static str, action: impl SetHeaderAction) -> Self {
                action.perform(Self::Custom(key), action)
            }
        }
    };
} Header! {
    1  CacheControl         = b"Cache-Control" | b"cache-control"
    2  Connection           = b"Connection" | b"connection"
    3  ContentDisposition   = b"Content-Disposition" | b"content-disposition"
    4  ContentEncoding      = b"Content-Encoding" | b"content-encoding"
    5  ContentLanguage      = b"Content-Language" | b"content-language"
    6  ContentLength        = b"Content-Length" | b"content-length"
    7  ContentLocation      = b"Content-Location" | b"content-location"
    8  ContentType          = b"Content-Type" | b"content-type"
    9  Date                 = b"Date" | b"date"
    10 Link                 = b"Link" | b"link"
    11 SecWebSocketProtocol = b"Sec-WebSocket-Protocol" | b"sec-websocket-protocol"
    12 SecWebSocketVersion  = b"Sec-WebSocket-Version" | b"sec-websocket-version"
    13 Trailer              = b"Trailer" | b"trailer"
    14 TransferEncoding     = b"Transfer-Encoding" | b"transfer-encoding"
    15 Upgrade              = b"Upgrade" | b"upgrade"
    16 Via                  = b"Via" | b"via"

    17 Accept                      = b"Accept" | b"accept",
    18 AcceptEncoding              = b"Accept-Encoding" | b"accept-encoding",
    19 AcceptLanguage              = b"Accept-Language" | b"accept-language",
    20 AccessControlRequestHeaders = b"Access-Control-Request-Headers" | b"access-control-request-headers",
    21 AccessControlRequestMethod  = b"Access-Control-Request-Method" | b"access-control-request-method",
    22 Authorization               = b"Authorization" | b"authorization",
    23 Cookie                      = b"Cookie" | b"cookie",
    24 Expect                      = b"Expect" | b"expect",
    25 Forwarded                   = b"Forwarded" | b"forwarded",
    26 From                        = b"From" | b"from",
    27 Host                        = b"Host" | b"host",
    28 IfMatch                     = b"If-Match" | b"if-match",
    29 IfModifiedSince             = b"If-Modified-Since" | b"if-modified-since",
    30 IfNoneMatch                 = b"If-None-Match" | b"rf-none-match",
    31 IfRange                     = b"If-Range" | b"if-range",
    32 IfUnmodifiedSince           = b"If-Unmodified-Since" | b"if-unmodified-since",
    33 MaxForwards                 = b"Max-Forwards" | b"max-forwards",
    34 Origin                      = b"Origin" | b"origin", 
    35 ProxyAuthorization          = b"Proxy-Authorization" | b"proxy-authorization",
    36 Range                       = b"Range" | b"range",
    37 Referer                     = b"Referer" | b"referer",
    38 SecWebSocketExtensions      = b"Sec-WebSocket-Extensions" | b"sec-websocket-extensions",
    39 SecWebSocketKey             = b"Sec-WebSocket-Key" | b"sec-websocket-key",
    40 TE,                         = b"TE" | b"te"
    41 UserAgent                   = b"User-Agent" | b"user-agent",
    42 UpgradeInsecureRequests     = b"Upgrade-Insecure-Requests" | b"upgrade-insecure-requests",

    43 AcceptRange                     = b"Accept-Range",
    44 AcceptRanges                    = b"Accept-Ranges",
    45 AccessControlAllowCredentials   = b"Access-Control-Allow-Credentials",
    46 AccessControlAllowHeaders       = b"Access-Control-Allow-Headers",
    47 AccessControlAllowMethods       = b"Access-Control-Allow-Methods",
    48 AccessControlAllowOrigin        = b"Access-Control-Allow-Origin",
    49 AccessControlExposeHeaders      = b"Access-Control-Expose-Headers",
    50 AccessControlMaxAge             = b"Access-Control-MaxAge",
    51 Age                             = b"Age",
    52 Allow                           = b"Allow",
    53 AltSvc                          = b"Alt-Svc",
    54 CacheStatus                     = b"Cache-Status",
    55 CDNCacheControl                 = b"CDN-Cache-Control",
    56 ContentRange                    = b"Content-Range",
    57 ContentSecurityPolicy           = b"Content-Security-Policy",
    58 ContentSecurityPolicyReportOnly = b"Content-Security-Policy",
    59 Etag                            = b"Etag",
    60 Expires                         = b"Expires",
    67 Location                        = b"Location",
    68 ProxyAuthenticate               = b"Proxy-Auhtneticate",
    69 ReferrerPolicy                  = b"Referrer-Policy",
    70 Refresh                         = b"Refresh",
    71 RetryAfter                      = b"Retry-After",
    72 SecWebSocketAccept              = b"Sec-Sert",
    73 Server                          = b"server",
    74 SetCookie                       = b"SetCookie",
    75 StrictTransportSecurity         = b"Strict-Transport-Security",
    76 Vary                            = b"Vary",
    77 XContentTypeOptions             = b"X-Content-Type-Options",
    78 XFrameOptions,                  = b"X-Frame-Options"
}


#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Header {
    /* entity */
    CacheControl,
    Connection,
    ContentDisposition,
    ContentEncoding,
    ContentLanguage,
    ContentLength,
    ContentLocation,
    ContentType,
    Date,
    Link,
    SecWebSocketProtocol,
    SecWebSocketVersion,
    Trailer,
    TransferEncoding,
    Upgrade,
    Via,

    /* response only */
    AcceptRange,
    AcceptRanges,
    AccessControlAllowCredentials,
    AccessControlAllowHeaders,
    AccessControlAllowMethods,
    AccessControlAllowOrigin,
    AccessControlExposeHeaders,
    AccessControlMaxAge,
    Age,
    Allow,
    AltSvc,
    CacheStatus,
    CDNCacheControl,
    ContentRange,
    ContentSecurityPolicy,
    ContentSecurityPolicyReportOnly,
    Etag,
    Expires,
    Location,
    ProxyAuthenticate,
    ReferrerPolicy,
    Refresh,
    RetryAfter,
    SecWebSocketAccept,
    Server,
    SetCookie,
    StrictTransportSecurity,
    Vary,
    XContentTypeOptions,
    XFrameOptions,

    /* request only */
    Accept,
    AcceptEncoding,
    AcceptLanguage,
    AccessControlRequestHeaders,
    AccessControlRequestMethod,
    Authorization,
    Cookie,
    Expect,
    Forwarded,
    From,
    Host,
    IfMatch,
    IfModifiedSince,
    IfNoneMatch,
    IfRange,
    IfUnmodifiedSince,
    MaxForwards,
    Origin,
    ProxyAuthorization,
    Range,
    Referer,
    SecWebSocketExtensions,
    SecWebSocketKey,
    TE,
    UserAgent,
    UpgradeInsecureRequests,

    Custom(&'static [u8]),
}
#[derive(Default)]
struct HeaderHasher {
    hash: usize
}
impl HeaderMap {
    pub fn new() -> Self {
        Self {
            map:  HashMap::default(),
            size: 2/* `\r\n` */,
        }
    }

    #[inline]
    pub fn insert(&mut self,
        key:   &'static str,
        value: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        let value = value.into();

        let (key_size, value_size) = (key.len(), value.len());

        match self.map.insert(key, value) {
            None      => {self.size += key_size + 2/* `: ` */ + value_size + 2/* `\r\n` */}
            Some(old) => {self.size -= old.len(); self.size += value_size}
        }

        self
    }

    #[inline]
    pub fn remove(&mut self, key: &'static str) -> &mut Self {
        if let Some(old) = self.map.remove(&key) {
            self.size -= key.len() + 2/* `: ` */ + old.len() + 2/* \r\n */;
        }
        
        self
    }

    #[inline]
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        macro_rules! push {
            ($buf:ident <- $bytes:expr) => {
                unsafe {
                    let (buf_len, bytes_len) = ($buf.len(), $bytes.len());
                    std::ptr::copy_nonoverlapping(
                        $bytes.as_ptr(),
                        <[u8]>::as_mut_ptr($buf).add(buf_len),
                        bytes_len
                    );
                    $buf.set_len(buf_len + bytes_len);
                }
            };
        }

        buf.reserve(self.size);

        for (k, v) in &self.map {
            push!(buf <- k.as_bytes());
            push!(buf <- b": ");
            push!(buf <- v.as_bytes());
            push!(buf <- b"\r\n");
        }
        push!(buf <- b"\r\n")
    }
}

impl HeaderHasher {
    #[inline(always)]
    fn add_to_hash(&mut self, i: usize) {
        #[cfg(target_pointer_width = "32")]
        const K: usize = 0x9e3779b9;
        #[cfg(target_pointer_width = "64")]
        const K: usize = 0x517cc1b727220a95;

        self.hash = self.hash.rotate_left(5).bitxor(i).wrapping_mul(K);
    }

    #[inline]
    fn write_fxhash(&mut self, mut bytes: &[u8]) {
        #[cfg(debug_assertions)] {
            assert!(size_of::<usize>() <= 8);
        }

        #[cfg(target_pointer_width = "32")]
        let read_usize = |bytes: &[u8]| u32::from_ne_bytes(bytes[..4].try_into().unwrap());
        #[cfg(target_pointer_width = "64")]
        let read_usize = |bytes: &[u8]| u64::from_ne_bytes(bytes[..8].try_into().unwrap());

        let mut hash = Self { hash: self.hash };
        while bytes.len() >= size_of::<usize>() {
            hash.add_to_hash(read_usize(bytes) as usize);
            bytes = &bytes[size_of::<usize>()..];
        }
        if (size_of::<usize>() > 4) && (bytes.len() >= 4) {
            hash.add_to_hash(u32::from_ne_bytes(bytes[..4].try_into().unwrap()) as usize);
            bytes = &bytes[4..];
        }
        if (size_of::<usize>() > 2) && bytes.len() >= 2 {
            hash.add_to_hash(u16::from_ne_bytes(bytes[..2].try_into().unwrap()) as usize);
            bytes = &bytes[2..];
        }
        if (size_of::<usize>() > 1) && bytes.len() >= 1 {
            hash.add_to_hash(bytes[0] as usize);
        }
        self.hash = hash.hash;
    }
}


impl Hasher for HeaderHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.hash = match bytes {
            /* entity */
            b"Cache-Control" | b"cache-control"                                   => 0,
            b"Connection" | b"connection"                                         => 1,
            b"Content-Disposition" | b"content-disposition"                       => 2,
            b"Content-Encoding" | b"content-encoding"                             => 3,
            b"Content-Language" | b"content-language"                             => 4,
            b"Content-Length" | b"content-length"                                 => 5,
            b"Content-Location" | b"content-location"                             => 6,
            b"Content-Type" | b"content-type"                                     => 7,
            b"Date" | b"date"                                                     => 8,
            b"Link" | b"link"                                                     => 9,
            b"Sec-WebSocket-Protocol" | b"sec-websocket-protocol"                 => 10,
            b"Sec-WebSocket-Version" | b"sec-websocket-version"                   => 11,
            b"Trailer" | b"trailer"                                               => 12,
            b"Transfer-Encoding" | b"transfer-encoding"                           => 13,
            b"Upgrade" | b"upgrade"                                               => 14,
            b"Via" | b"via"                                                       => 15,

            /* response only */
            b"Accept-Range"                                                       => 16,
            b"Accept-Ranges"                                                      => 17,
            b"Access-Control-Allow-Credentials"                                   => 18,
            b"Access-Control-Allow-Headers"                                       => 19,
            b"Access-Control-Allow-Methods"                                       => 20,
            b"Access-Control-Allow-Origin"                                        => 21,
            b"Access-Control-Expose-Headers"                                      => 22,
            b"Access-Control-Max-Age"                                             => 23,
            b"Age"                                                                => 24,
            b"Allow"                                                              => 25,
            b"Alt-Svc"                                                            => 26,
            b"CacheStatus"                                                        => 27,
            b"CDNCacheControl"                                                    => 28,
            b"Content-Range"                                                      => 29,
            b"Content-Security-Policy"                                            => 30,
            b"Content-Security-Policy-Report-Only"                                => 31,
            b"Etag"                                                               => 32,
            b"Expires"                                                            => 33,
            b"Location"                                                           => 34,
            b"Proxy-Authenticate"                                                 => 35,
            b"Referrer-Policy"                                                    => 36,
            b"Refresh"                                                            => 37,
            b"Retry-After"                                                        => 38,
            b"Sec-WebSocket-Accept"                                               => 39,
            b"Server"                                                             => 40,
            b"Set-Cookie"                                                         => 41,
            b"Strict-Transport-Security"                                          => 42,
            b"Vary"                                                               => 43,
            b"X-Content-Type-Options"                                             => 44,
            b"X-Frame-Options"                                                    => 45,

            /* request only */
            b"Accept" | b"accept"                                                 => 46,
            b"Accept-Encoding" | b"accept-encoding"                               => 47,
            b"Accept-Language" | b"accept-language"                               => 48,
            b"Access-Control-Request-Headers" | b"access-control-request-headers" => 49,
            b"Access-Control-Request-Method" | b"access-control-request-method"   => 50,
            b"Authorization" | b"authorization"                                   => 51,
            b"Cookie" | b"cookie"                                                 => 52,
            b"Expect" | b"expect"                                                 => 53,
            b"Forwarded" | b"forwarded"                                           => 54,
            b"From" | b"from"                                                     => 55,
            b"Host" | b"host"                                                     => 56,
            b"If-Match" | b"if-match"                                             => 57,
            b"If-Modified-Since" | b"if-modified-since"                           => 58,
            b"If-None-Match" | b"if-none-match"                                   => 59,
            b"If-Range" | b"if-range"                                             => 60,
            b"If-Unmodified-Since" | b"if-unmodified-since"                       => 61,
            b"Max-Forwards" | b"max-forwards"                                     => 62,
            b"Origin" | b"origin"                                                 => 63,
            b"Proxy-Authorization" | b"proxy-authorization"                       => 64,
            b"Range" | b"range"                                                   => 65,
            b"Referer" | b"referer"                                               => 66,
            b"Sec-WebSocket-Extensions" | b"sec-websocket-extensions"             => 67,
            b"Sec-WebSocket-Key" | b"sec-websocket-key"                           => 68,
            b"TE" | b"te"                                                         => 69,
            b"User-Agent" | b"user-agent"                                         => 70,
            b"Upgrade-Insecure-Requests" | b"upgrade-insecure-requests"           => 71,
            
            custom => return self.write_fxhash(custom)
        }
    }
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.hash as _
    }
}




#[cfg(test)]
#[test] fn hash_header() {
    let mut h = HeaderHasher::default();
    h.write(b"Content-Type");
    assert_eq!(h.finish(), 7);

    let mut h = HeaderHasher::default();
    h.write(b"Content-Security-Policy-Report-Only");
    assert_eq!(h.finish(), 31);
}

#[cfg(test)]
#[test] fn edit_map() {
    let mut h = HeaderMap::new();

    h
        .insert("Content-Type", "application/json")
        .insert("Content-Length", r#"{"name": "ohkami", "type": "web framework"}"#)
        .insert("Access-Control-Allow-Credentials", "true")
        .insert("Access-Control-Allow-Headers", "X-Custom-Header,Upgrade-Insecure-Requests")
        .insert("Access-Control-Allow-Origin", "https://foo.bar.org")
        .insert("Access-Control-Max-Age", "86400")
        .insert("Vary", "Origin")
        .insert("Server", "ohkami")
        .insert("Connection", "Keep-Alive")
        .insert("Date", "Wed, 21 Oct 2015 07:28:00 GMT")
        .insert("Alt-Svc", "h2=\":433\"; ma=2592000")
        .insert("Proxy-Authenticate", "Basic realm=\"Access to the internal site\"")
        .insert("Referrer-Policy", "same-origin")
        .insert("X-Frame-Options", "DEBY")
        .insert("X-Custom-Data", "something")
        .insert("My-Custom-Header", "Anything");

    h
        .remove("Alt-Svc")
        .remove("Referrer-Policy")
        .remove("Referer-Policy")
        .remove("X-Custom-Data")
        .remove("Connection");

    assert_eq!(h.map, HashMap::from_iter([
        ("Content-Type", Cow::Borrowed("application/json")),
        ("Content-Length", Cow::Borrowed(r#"{"name": "ohkami", "type": "web framework"}"#)),
        ("Access-Control-Allow-Credentials", Cow::Borrowed("true")),
        ("Access-Control-Allow-Headers", Cow::Borrowed("X-Custom-Header,Upgrade-Insecure-Requests")),
        ("Access-Control-Allow-Origin", Cow::Borrowed("https://foo.bar.org")),
        ("Access-Control-Max-Age", Cow::Borrowed("86400")),
        ("Vary", Cow::Borrowed("Origin")),
        ("Server", Cow::Borrowed("ohkami")),
        // ("Connection", Cow::Borrowed("Keep-Alive")),
        ("Date", Cow::Borrowed("Wed, 21 Oct 2015 07:28:00 GMT")),
        // ("Alt-Svc", Cow::Borrowed("h2=\":433\"; ma=2592000")),
        ("Proxy-Authenticate", Cow::Borrowed("Basic realm=\"Access to the internal site\"")),
        // ("Referrer-Policy", Cow::Borrowed("same-origin")),
        ("X-Frame-Options", Cow::Borrowed("DEBY")),
        // ("X-Custom-Data", Cow::Borrowed("something")),
        ("My-Custom-Header", Cow::Borrowed("Anything")),
    ]));
}
