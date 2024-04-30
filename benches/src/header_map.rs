// use ohkami_lib::CowSlice;
// use rustc_hash::FxHasher;
use core::hash::{BuildHasherDefault, Hasher, Hash};
use std::{borrow::Cow, collections::HashMap};


#[derive(Default)]
struct HeaderHasher {
    hash: usize,
}
impl HeaderHasher {
    #[inline(always)]
    fn add_to_hash(&mut self, i: usize) {
        use core::ops::BitXor;

        #[cfg(target_pointer_width = "32")]
        const K: usize = 0x9e3779b9;
        #[cfg(target_pointer_width = "64")]
        const K: usize = 0x517cc1b727220a95;
        
        self.hash = self.hash.rotate_left(5).bitxor(i).wrapping_mul(K);
    }
}
impl Hasher for HeaderHasher {
    #[inline]
    fn write(&mut self, mut bytes: &[u8]) {
        #[cfg(debug_assertions)]
        assert!(::core::mem::size_of::<usize>() <= 8);

        #[cfg(target_pointer_width = "32")]
        let read_usize = |bytes: &[u8]| u32::from_ne_bytes(bytes[..4].try_into().unwrap());
        #[cfg(target_pointer_width = "64")]
        let read_usize = |bytes: &[u8]| u64::from_ne_bytes(bytes[..8].try_into().unwrap());

        let mut hash = Self { hash: self.hash };
        while bytes.len() >= ::core::mem::size_of::<usize>() {
            hash.add_to_hash(read_usize(bytes) as usize);
            bytes = &bytes[::core::mem::size_of::<usize>()..];
        }
        if (::core::mem::size_of::<usize>() > 4) && (bytes.len() >= 4) {
            hash.add_to_hash(u32::from_ne_bytes(bytes[..4].try_into().unwrap()) as usize);
            bytes = &bytes[4..];
        }
        if (::core::mem::size_of::<usize>() > 2) && bytes.len() >= 2 {
            hash.add_to_hash(u16::from_ne_bytes(bytes[..2].try_into().unwrap()) as usize);
            bytes = &bytes[2..];
        }
        if (::core::mem::size_of::<usize>() > 1) && bytes.len() >= 1 {
            hash.add_to_hash(bytes[0] as usize);
        }
        self.hash = hash.hash;
    }

    #[inline(always)]
    fn write_usize(&mut self, i: usize) {
        self.hash = i
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash as _
    }
}

macro_rules! HeaderMap {
    ($( $index:literal : $name:ident = $name_pascal:literal $(| $name_lower:literal)?)*) => {
        pub struct HeaderMap {
            map:  HashMap<Header, Cow<'static, str>, BuildHasherDefault<HeaderHasher>>,
            size: usize
        }


        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
        #[allow(non_camel_case_types)]
        pub enum Header {
            $(
                $name,
            )*
            custom(&'static [u8])
        }
        impl Header {
            #[inline(always)]
            const fn as_bytes(&self) -> &'static [u8] {
                match self {
                    $(
                        Self::$name => $name_pascal,
                    )*
                    Self::custom(bytes) => bytes
                }
            }
        }
        impl Hash for Header {
            fn hash<H: Hasher>(&self, state: &mut H) {
                state.write_usize(match self {
                    $(
                        Self::$name => $index,
                    )*
                    Self::custom(bytes) => return state.write(bytes)
                })
            }
        }        


        impl HeaderMap {
            pub fn set(&mut self) -> SetHeader<'_> {
                SetHeader(self)
            }
        }

        pub struct SetHeader<'s>(&'s mut HeaderMap);

        pub trait SetHeaderAction<'s> {
            fn perform(self, key: Header, set: SetHeader<'s>) -> SetHeader<'s>;
        } const _: () = {
            impl<'s> SetHeaderAction<'s> for Option<()> {
                #[inline]
                fn perform(self, key: Header, set: SetHeader<'s>) -> SetHeader<'s> {
                    set.0.remove(key);
                    set
                }
            }

            impl<'s> SetHeaderAction<'s> for &'static str {
                #[inline(always)]
                fn perform(self, key: Header, set: SetHeader<'s>) -> SetHeader<'s> {
                    set.0.insert(key, self);
                    set
                }
            }
            impl<'s> SetHeaderAction<'s> for &String {
                fn perform(self, key: Header, set: SetHeader<'s>) -> SetHeader<'s> {
                    set.0.insert(key, self.clone());
                    set
                }
            }
            impl<'s> SetHeaderAction<'s> for String {
                #[inline(always)]
                fn perform(self, key: Header, set: SetHeader<'s>) -> SetHeader<'s> {
                    set.0.insert(key, self);
                    set
                }
            }
        };

        #[allow(non_snake_case)]
        impl<'s> SetHeader<'s> {
            $(
                #[inline(always)]
                pub fn $name(self, action: impl SetHeaderAction<'s>) -> Self {
                    action.perform(Header::$name, self)
                }
            )*
            #[inline]
            pub fn custom(self, key: &'static str, action: impl SetHeaderAction<'s>) -> Self {
                action.perform(Header::custom(key.as_bytes()), self)
            }
        }
    };
} HeaderMap! {
    1 : CacheControl         = b"Cache-Control" | b"cache-control"
    2 : Connection           = b"Connection" | b"connection"
    3 : ContentDisposition   = b"Content-Disposition" | b"content-disposition"
    4 : ContentEncoding      = b"Content-Encoding" | b"content-encoding"
    5 : ContentLanguage      = b"Content-Language" | b"content-language"
    6 : ContentLength        = b"Content-Length" | b"content-length"
    7 : ContentLocation      = b"Content-Location" | b"content-location"
    8 : ContentType          = b"Content-Type" | b"content-type"
    9 : Date                 = b"Date" | b"date"
    10: Link                 = b"Link" | b"link"
    11: SecWebSocketProtocol = b"Sec-WebSocket-Protocol" | b"sec-websocket-protocol"
    12: SecWebSocketVersion  = b"Sec-WebSocket-Version" | b"sec-websocket-version"
    13: Trailer              = b"Trailer" | b"trailer"
    14: TransferEncoding     = b"Transfer-Encoding" | b"transfer-encoding"
    15: Upgrade              = b"Upgrade" | b"upgrade"
    16: Via                  = b"Via" | b"via"

    17: Accept                      = b"Accept" | b"accept"
    18: AcceptEncoding              = b"Accept-Encoding" | b"accept-encoding"
    19: AcceptLanguage              = b"Accept-Language" | b"accept-language"
    20: AccessControlRequestHeaders = b"Access-Control-Request-Headers" | b"access-control-request-headers"
    21: AccessControlRequestMethod  = b"Access-Control-Request-Method" | b"access-control-request-method"
    22: Authorization               = b"Authorization" | b"authorization"
    23: Cookie                      = b"Cookie" | b"cookie"
    24: Expect                      = b"Expect" | b"expect"
    25: Forwarded                   = b"Forwarded" | b"forwarded"
    26: From                        = b"From" | b"from"
    27: Host                        = b"Host" | b"host"
    28: IfMatch                     = b"If-Match" | b"if-match"
    29: IfModifiedSince             = b"If-Modified-Since" | b"if-modified-since"
    30: IfNoneMatch                 = b"If-None-Match" | b"rf-none-match"
    31: IfRange                     = b"If-Range" | b"if-range"
    32: IfUnmodifiedSince           = b"If-Unmodified-Since" | b"if-unmodified-since"
    33: MaxForwards                 = b"Max-Forwards" | b"max-forwards"
    34: Origin                      = b"Origin" | b"origin" 
    35: ProxyAuthorization          = b"Proxy-Authorization" | b"proxy-authorization"
    36: Range                       = b"Range" | b"range"
    37: Referer                     = b"Referer" | b"referer"
    38: SecWebSocketExtensions      = b"Sec-WebSocket-Extensions" | b"sec-websocket-extensions"
    39: SecWebSocketKey             = b"Sec-WebSocket-Key" | b"sec-websocket-key"
    40: TE                          = b"TE" | b"te"
    41: UserAgent                   = b"User-Agent" | b"user-agent"
    42: UpgradeInsecureRequests     = b"Upgrade-Insecure-Requests" | b"upgrade-insecure-requests"

    43: AcceptRange                     = b"Accept-Range"
    44: AcceptRanges                    = b"Accept-Ranges"
    45: AccessControlAllowCredentials   = b"Access-Control-Allow-Credentials"
    46: AccessControlAllowHeaders       = b"Access-Control-Allow-Headers"
    47: AccessControlAllowMethods       = b"Access-Control-Allow-Methods"
    48: AccessControlAllowOrigin        = b"Access-Control-Allow-Origin"
    49: AccessControlExposeHeaders      = b"Access-Control-Expose-Headers"
    50: AccessControlMaxAge             = b"Access-Control-MaxAge"
    51: Age                             = b"Age"
    52: Allow                           = b"Allow"
    53: AltSvc                          = b"Alt-Svc"
    54: CacheStatus                     = b"Cache-Status"
    55: CDNCacheControl                 = b"CDN-Cache-Control"
    56: ContentRange                    = b"Content-Range"
    57: ContentSecurityPolicy           = b"Content-Security-Policy"
    58: ContentSecurityPolicyReportOnly = b"Content-Security-Policy"
    59: Etag                            = b"Etag"
    60: Expires                         = b"Expires"
    67: Location                        = b"Location"
    68: ProxyAuthenticate               = b"Proxy-Auhtneticate"
    69: ReferrerPolicy                  = b"Referrer-Policy"
    70: Refresh                         = b"Refresh"
    71: RetryAfter                      = b"Retry-After"
    72: SecWebSocketAccept              = b"Sec-Sert"
    73: Server                          = b"server"
    74: SetCookie                       = b"SetCookie"
    75: StrictTransportSecurity         = b"Strict-Transport-Security"
    76: Vary                            = b"Vary"
    77: XContentTypeOptions             = b"X-Content-Type-Options"
    78: XFrameOptions                   = b"X-Frame-Options"
}

impl HeaderMap {
    pub fn new() -> Self {
        Self {
            map:  HashMap::with_capacity_and_hasher(32, BuildHasherDefault::<HeaderHasher>::default()),
            size: 2/* `\r\n` */,
        }
    }

    #[inline]
    fn insert(&mut self,
        key:   Header,
        value: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        let value = value.into();

        let (key_size, value_size) = (key.as_bytes().len(), value.len());

        match self.map.insert(key, value) {
            None      => {self.size += key_size + 2/* `: ` */ + value_size + 2/* `\r\n` */}
            Some(old) => {self.size -= old.len(); self.size += value_size}
        }

        self
    }

    #[inline]
    fn remove(&mut self, key: Header) -> &mut Self {
        if let Some(old) = self.map.remove(&key) {
            self.size -= key.as_bytes().len() + 2/* `: ` */ + old.len() + 2/* \r\n */;
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




#[cfg(test)]
// #[test]
fn _edit_map() {
    let mut h = HeaderMap::new();

    h.set()
        .ContentType("application/json")
        .ContentLength("42")
        .AccessControlAllowCredentials("true")
        .AccessControlAllowHeaders("X-Custom-Header,Upgrade-Insecure-Requests")
        .AccessControlAllowOrigin("https://foo.bar.org")
        .AccessControlMaxAge("86400")
        .Vary("Origin")
        .Server("ohkami")
        .Connection("Keep-Alive")
        .Date("Wed, 21 Oct 2015 07:28:00 GMT")
        .AltSvc("h2=\":433\"; ma=2592000")
        .ProxyAuthenticate("Basic realm=\"Access to the internal site\"")
        .ReferrerPolicy("same-origin")
        .XFrameOptions("DEBY")
        .custom("X-Custom-Data", "something")
        .custom("My-Custom-Header", "Anything")
    ;

    h.set()
        .AltSvc(None)
        .ReferrerPolicy(None)
        .Connection(None)
        .custom("X-Custom-Data", None)
    ;

    assert_eq!(h.map, HashMap::from_iter([
        (Header::ContentType, Cow::Borrowed("application/json")),
        (Header::ContentLength, Cow::Borrowed("42")),
        (Header::AccessControlAllowCredentials, Cow::Borrowed("true")),
        (Header::AccessControlAllowHeaders, Cow::Borrowed("X-Custom-Header,Upgrade-Insecure-Requests")),
        (Header::AccessControlAllowOrigin, Cow::Borrowed("https://foo.bar.org")),
        (Header::AccessControlMaxAge, Cow::Borrowed("86400")),
        (Header::Vary, Cow::Borrowed("Origin")),
        (Header::Server, Cow::Borrowed("ohkami")),
        // ("Connection", Cow::Borrowed("Keep-Alive")),
        (Header::Date, Cow::Borrowed("Wed, 21 Oct 2015 07:28:00 GMT")),
        // ("Alt-Svc", Cow::Borrowed("h2=\":433\"; ma=2592000")),
        (Header::ProxyAuthenticate, Cow::Borrowed("Basic realm=\"Access to the internal site\"")),
        // ("Referrer-Policy", Cow::Borrowed("same-origin")),
        (Header::XFrameOptions, Cow::Borrowed("DEBY")),
        // ("X-Custom-Data", Cow::Borrowed("something")),
        // (Header::custom(b"My-Custom-Header"), Cow::Borrowed("Anything")),
    ]));
}
