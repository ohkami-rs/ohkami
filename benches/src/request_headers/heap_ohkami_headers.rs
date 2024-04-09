use rustc_hash::FxHashMap;
use ohkami_lib::{CowSlice, Slice};


pub struct HeapOhkamiHeaders {
    standard: Box<[Option<CowSlice>; N_CLIENT_HEADERS]>,
    custom:   Option<Box<FxHashMap<Slice, CowSlice>>>,
}
impl HeapOhkamiHeaders {
    pub fn new() -> Self {
        HeapOhkamiHeaders {
            standard: Box::new([
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None,
            ]),
            custom: None
        }
    }

    #[inline(always)] pub fn insert(&mut self, name: Header, value: CowSlice) {
        unsafe {*self.standard.get_unchecked_mut(name as usize) = Some(value)}
    }
    #[inline] pub fn remove(&mut self, name: Header) {
        unsafe {*self.standard.get_unchecked_mut(name as usize) = None}
    }
    #[inline] pub fn get(&self, name: Header) -> Option<&str> {
        match unsafe {self.standard.get_unchecked(name as usize)} {
            Some(v) => Some(std::str::from_utf8(
                unsafe {v.as_bytes()}
            ).expect("Header value is not UTF-8")),
            None => None,
        }
    }

    #[inline] pub fn insert_custom(&mut self, name: Slice, value: CowSlice) {
        match &mut self.custom {
            Some(c) => {c.insert(name, value);}
            None => self.custom = Some(Box::new(FxHashMap::from_iter([
                (name, value)
            ])))
        }
    }
    #[inline] pub fn remove_custom(&mut self, name: Slice) {
        if let Some(c) = &mut self.custom {
            c.remove(&name);
        }
    }

    pub fn set<'s>(&'s mut self) -> SetHeaders<'s> {
        SetHeaders(self)
    }
}

pub struct SetHeaders<'s>(&'s mut HeapOhkamiHeaders);

pub trait SetHeaderAction<'a> {
    fn perform(self, set: SetHeaders<'a>, key: Header) -> SetHeaders<'a>;
} const _: () = {
    // remove
    impl<'set> SetHeaderAction<'set> for Option<()> {
        fn perform(self, set_headers: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set_headers.0.remove(key);
            set_headers
        }
    }

    // insert
    impl<'set> SetHeaderAction<'set> for &'static str {
        #[inline] fn perform(self, set_headers: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set_headers.0.insert(key, CowSlice::Ref(unsafe {Slice::from_bytes(self.as_bytes())}));
            set_headers
        }
    }
    impl<'set> SetHeaderAction<'set> for String {
        #[inline] fn perform(self, set_headers: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set_headers.0.insert(key, CowSlice::Own(self.into_bytes()));
            set_headers
        }
    }
};

pub trait SetCustomHeaderAction<'a> {
    fn perform(self, set: SetHeaders<'a>, key: &'static str) -> SetHeaders<'a>;
} const _: () = {
    // remove
    impl<'set> SetCustomHeaderAction<'set> for Option<()> {
        fn perform(self, set_headers: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set_headers.0.remove_custom(unsafe {Slice::from_bytes(key.as_bytes())});
            set_headers
        }
    }

    // insert
    impl<'set> SetCustomHeaderAction<'set> for &'static str {
        #[inline] fn perform(self, set_headers: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set_headers.0.insert_custom(unsafe {Slice::from_bytes(key.as_bytes())}, CowSlice::Ref(unsafe {Slice::from_bytes(self.as_bytes())}));
            set_headers
        }
    }
    impl<'set> SetCustomHeaderAction<'set> for String {
        #[inline] fn perform(self, set_headers: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set_headers.0.insert_custom(unsafe {Slice::from_bytes(key.as_bytes())}, CowSlice::Own(self.into_bytes()));
            set_headers
        }
    }
};

macro_rules! Header {
    ($N:literal; $(
        $konst:ident: $name_bytes:literal | $lower_case:literal $(| $other_pattern:literal)* ,
    )*) => {
        const N_CLIENT_HEADERS: usize = $N;
        // const CLIENT_HEADERS: [Header; N_CLIENT_HEADERS] = [ $( Header::$konst ),* ];

        #[derive(Debug, PartialEq)]
        pub enum Header {
            $( $konst, )*
        }
        impl Header {
            #[inline] pub const fn as_str(&self) -> &'static str {
                match self {
                    $(
                        Self::$konst => unsafe {std::str::from_utf8_unchecked($name_bytes)},
                    )*
                }
            }
            #[inline] pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
                match bytes {
                    $(
                        $name_bytes | $lower_case $(| $other_pattern)* => Some(Self::$konst),
                    )*
                    _ => None
                }
            }
        }

        impl<T: AsRef<[u8]>> PartialEq<T> for Header {
            fn eq(&self, other: &T) -> bool {
                self.as_str().as_bytes().eq_ignore_ascii_case(other.as_ref())
            }
        }

        #[allow(non_snake_case)]
        impl<'set> SetHeaders<'set> {
            $(
                pub fn $konst(self, action: impl SetHeaderAction<'set>) -> Self {
                    action.perform(self, Header::$konst)
                }
            )*

            pub fn custom(self, name: &'static str, action: impl SetCustomHeaderAction<'set>) -> Self {
                if self.0.custom.is_none() {
                    self.0.custom = Some(Box::new(FxHashMap::default()));
                }
                action.perform(self, name)
            }
        }

    };
} Header! {42;
    Accept:                      b"Accept" | b"accept",
    AcceptEncoding:              b"Accept-Encoding" | b"accept-encoding",
    AcceptLanguage:              b"Accept-Language" | b"accept-language",
    AccessControlRequestHeaders: b"Access-Control-Request-Headers" | b"access-control-request-headers",
    AccessControlRequestMethod:  b"Access-Control-Request-Method" | b"access-control-request-method",
    Authorization:               b"Authorization" | b"authorization",
    CacheControl:                b"Cache-Control" | b"cache-control",
    Connection:                  b"Connection" | b"connection",
    ContentDisposition:          b"Content-Disposition" | b"content-disposition",
    ContentEncoding:             b"Content-Encoding" | b"content-encoding",
    ContentLanguage:             b"Content-Language" | b"content-language",
    ContentLength:               b"Content-Length" | b"content-length",
    ContentLocation:             b"Content-Location" | b"content-location",
    ContentType:                 b"Content-Type" | b"content-type",
    Cookie:                      b"Cookie" | b"cookie",
    Date:                        b"Date" | b"date",
    Expect:                      b"Expect" | b"expect",
    Forwarded:                   b"Forwarded" | b"forwarded",
    From:                        b"From" | b"from",
    Host:                        b"Host" | b"host",
    IfMatch:                     b"If-Match" | b"if-match",
    IfModifiedSince:             b"If-Modified-Since" | b"if-modified-since",
    IfNoneMatch:                 b"If-None-Match" | b"if-none-match",
    IfRange:                     b"If-Range" | b"if-range",
    IfUnmodifiedSince:           b"If-Unmodified-Since" | b"if-unmodified-since",
    Link:                        b"Link" | b"link",
    MaxForwards:                 b"Max-Forwards" | b"max-forwards",
    Origin:                      b"Origin" | b"origin",
    ProxyAuthorization:          b"Proxy-Authorization" | b"proxy-authorization",
    Range:                       b"Range" | b"range",
    Referer:                     b"Referer" | b"referer",
    SecWebSocketExtensions:      b"Sec-WebSocket-Extensions" | b"sec-websocket-extensions",
    SecWebSocketKey:             b"Sec-WebSocket-Key" | b"sec-websocket-key",
    SecWebSocketProtocol:        b"Sec-WebSocket-Protocol" | b"sec-websocket-protocol",
    SecWebSocketVersion:         b"Sec-WebSocket-Version" | b"sec-websocket-version",
    TE:                          b"TE" | b"te",
    Trailer:                     b"Trailer" | b"trailer",
    TransferEncoding:            b"Transfer-Encoding" | b"transfer-encoding",
    UserAgent:                   b"User-Agent" | b"user-agent",
    Upgrade:                     b"Upgrade" | b"upgrade",
    UpgradeInsecureRequests:     b"Upgrade-Insecure-Requests" | b"upgrade-insecure-requests",
    Via:                         b"Via" | b"via",
}
