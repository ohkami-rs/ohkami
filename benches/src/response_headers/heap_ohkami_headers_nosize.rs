use std::borrow::Cow;
use rustc_hash::FxHashMap;


pub struct HeapOhkamiHeadersWithoutSize {
    standard: Box<[Option<Cow<'static, str>>; N_SERVER_HEADERS]>,
    custom:   Option<Box<FxHashMap<&'static str, Cow<'static, str>>>>,
}

impl HeapOhkamiHeadersWithoutSize {
    pub fn new() -> Self {
        Self {
            standard: Box::new([
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None, None, None, None,
                None, None, None
            ]),
            custom: None,
        }
    }

    pub fn set<'s>(&'s mut self) -> SetHeaders<'s> {
        SetHeaders(self)
    }
}

pub struct SetHeaders<'s>(&'s mut HeapOhkamiHeadersWithoutSize);

pub trait HeaderAction<'action> {
    fn perform(self, set_headers: SetHeaders<'action>, key: Header) -> SetHeaders<'action>;
} const _: () = {
    // remove
    impl<'a> HeaderAction<'a> for Option<()> {
        #[inline] fn perform(self, set_headers: SetHeaders<'a>, key: Header) -> SetHeaders<'a> {
            set_headers.0.remove(key);
            set_headers
        }
    }

    // insert
    impl<'a> HeaderAction<'a> for &'static str {
        #[inline] fn perform(self, set_headers: SetHeaders<'a>, key: Header) -> SetHeaders<'a> {
            set_headers.0.insert(key, Cow::Borrowed(self));
            set_headers
        }
    }
    impl<'a> HeaderAction<'a> for String {
        #[inline] fn perform(self, set_headers: SetHeaders<'a>, key: Header) -> SetHeaders<'a> {
            set_headers.0.insert(key, Cow::Owned(self));
            set_headers
        }
    }
    impl<'a> HeaderAction<'a> for std::borrow::Cow<'static, str> {
        fn perform(self, set_headers: SetHeaders<'a>, key: Header) -> SetHeaders<'a> {
            set_headers.0.insert(key, self);
            set_headers
        }
    }
};

pub trait CustomHeadersAction<'action> {
    fn perform(self, set: SetHeaders<'action>, key: &'static str) -> SetHeaders<'action>;
} const _: () = {
    // remove
    impl<'a> CustomHeadersAction<'a> for Option<()> {
        #[inline]
        fn perform(self, set: SetHeaders<'a>, key: &'static str) -> SetHeaders<'a> {
            if let Some(c) = &mut set.0.custom {
                c.remove(&key);
            }
            set
        }
    }

    // insert
    impl<'a> CustomHeadersAction<'a> for &'static str {
        #[inline(always)]
        fn perform(self, set: SetHeaders<'a>, key: &'static str) -> SetHeaders<'a> {
            match &mut set.0.custom {
                None => {
                    set.0.custom = Some(Box::new(FxHashMap::from_iter([(key, Cow::Borrowed(self))])));
                }
                Some(c) => {
                    c.insert(key, Cow::Borrowed(self));
                }
            }
            set
        }
    }
    impl<'a> CustomHeadersAction<'a> for String {
        #[inline(always)]
        fn perform(self, set: SetHeaders<'a>, key: &'static str) -> SetHeaders<'a> {
            match &mut set.0.custom {
                None => {
                    set.0.custom = Some(Box::new(FxHashMap::from_iter([(key, Cow::Owned(self))])));
                }
                Some(c) => {
                    c.insert(key, Cow::Owned(self));
                }
            }
            set
        }
    }
};


macro_rules! Header {
    ($N:literal; $( $konst:ident: $name_bytes:literal, )*) => {
        pub(crate) const N_SERVER_HEADERS: usize = $N;
        pub(crate) const SERVER_HEADERS: [Header; N_SERVER_HEADERS] = [ $( Header::$konst ),* ];

        #[derive(Debug, PartialEq, Clone, Copy)]
        pub enum Header {
            $( $konst, )*
        }

        impl Header {
            #[inline] pub const fn as_bytes(&self) -> &'static [u8] {
                match self {
                    $(
                        Self::$konst => $name_bytes,
                    )*
                }
            }
            #[inline] pub const fn as_str(&self) -> &'static str {
                unsafe {std::str::from_utf8_unchecked(self.as_bytes())}
            }

            pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
                match bytes {
                    $(
                        $name_bytes => Some(Self::$konst),
                    )*
                    _ => None
                }
            }
        }

        impl<T: AsRef<[u8]>> PartialEq<T> for Header {
            fn eq(&self, other: &T) -> bool {
                self.as_bytes().eq_ignore_ascii_case(other.as_ref())
            }
        }

        #[allow(non_snake_case)]
        impl<'set> SetHeaders<'set> {
            $(
                pub fn $konst(self, action: impl HeaderAction<'set>) -> Self {
                    action.perform(self, Header::$konst)
                }
            )*

            pub fn custom(self, name: &'static str, action: impl CustomHeadersAction<'set>) -> Self {
                action.perform(self, name)
            }
        }
    };
} Header! {45;
    AcceptRanges:                    b"Accept-Ranges",
    AccessControlAllowCredentials:   b"Access-Control-Allow-Credentials",
    AccessControlAllowHeaders:       b"Access-Control-Allow-Headers",
    AccessControlAllowMethods:       b"Access-Control-Allow-Methods",
    AccessControlAllowOrigin:        b"Access-Control-Allow-Origin",
    AccessControlExposeHeaders:      b"Access-Control-Expose-Headers",
    AccessControlMaxAge:             b"Access-Control-Max-Age",
    Age:                             b"Age",
    Allow:                           b"Allow",
    AltSvc:                          b"Alt-Svc",
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


impl HeapOhkamiHeadersWithoutSize {
    #[inline]
    fn insert(&mut self, key: Header, value: impl Into<Cow<'static, str>>) {
        unsafe {*self.standard.get_unchecked_mut(key as usize) = Some(value.into())}
    }

    #[inline]
    fn remove(&mut self, key: Header) {
        unsafe {*self.standard.get_unchecked_mut(key as usize) = None}
    }

    pub fn write_to(&self, buf: &mut Vec<u8>) {
        buf.reserve(256);
        for h in unsafe {SERVER_HEADERS.get_unchecked(1..)} {
            if let Some(v) = unsafe {self.standard.get_unchecked(*h as usize)} {
                buf.extend_from_slice(h.as_bytes());
                buf.extend_from_slice(b": ");
                buf.extend_from_slice(v.as_bytes());
                buf.extend_from_slice(b"\r\n");
            }
        }
        if let Some(c) = &self.custom {
            for (k, v) in &**c {
                buf.extend_from_slice(k.as_bytes());
                buf.extend_from_slice(b": ");
                buf.extend_from_slice(v.as_bytes());
                buf.extend_from_slice(b"\r\n");
            }
        }
        buf.extend_from_slice(b"\r\n");
    }

    pub fn write_standards_to(&self, buf: &mut Vec<u8>) {
        buf.reserve(256);
        for h in unsafe {SERVER_HEADERS.get_unchecked(1..)} {
            if let Some(v) = unsafe {self.standard.get_unchecked(*h as usize)} {
                buf.extend_from_slice(h.as_bytes());
                buf.extend_from_slice(b": ");
                buf.extend_from_slice(v.as_bytes());
                buf.extend_from_slice(b"\r\n");
            }
        }
        buf.extend_from_slice(b"\r\n");
    }
}
