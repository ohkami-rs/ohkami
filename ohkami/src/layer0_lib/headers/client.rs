use std::borrow::Cow;
use crate::layer0_lib::{CowSlice, Slice};


pub struct Headers {
    values: [Value; N_CLIENT_HEADERS],
}
pub struct Value(
    Option<CowSlice>,
); impl Value {
    pub fn as_str(&self) -> &str {
        match &self.0 {
            Some(cows) => std::str::from_utf8(unsafe {cows.as_bytes()}).expect("Header value is not UTF-8"),
            None       => "",
        }
    }
    pub fn append(&mut self, value: impl Into<Cow<'static, str>>) {
        let value: Cow<'static, str> = value.into();
        match &mut self.0 {
            Some(CowSlice::Own(vec)) => {
                vec.push(b',');
                vec.extend_from_slice(value.as_bytes());
            }
            Some(CowSlice::Ref(slice)) => {
                let mut this = unsafe{slice.as_bytes()}.to_vec();
                this.push(b',');
                this.extend_from_slice(value.as_bytes());
                self.0 = Some(CowSlice::Own(this))
            }
            None => self.0 = Some(match value {
                Cow::Borrowed(static_str) => CowSlice::Ref(unsafe {Slice::from_bytes(static_str.as_bytes())}),
                Cow::Owned(string)        => CowSlice::Own(string.into_bytes()),
            })
        };
    }
    pub fn replace(&mut self, new_value: impl Into<Cow<'static, str>>) {
        self.0 = Some(match new_value.into() {
            Cow::Borrowed(static_str) => CowSlice::Ref(unsafe {Slice::from_bytes(static_str.as_bytes())}),
            Cow::Owned(string)        => CowSlice::Own(string.into_bytes()),
        })
    }
}

pub struct SetHeaders<'set>(
    &'set mut Headers
); impl Headers {
    pub(crate) fn set(&mut self) -> SetHeaders<'_> {
        SetHeaders(self)
    }
}
pub trait HeaderAction<'set> {
    fn perform(self, set_headers: SetHeaders<'set>, key: Header) -> SetHeaders<'set>;
} const _: () = {
    // remove
    impl<'set> HeaderAction<'set> for Option<()> {
        fn perform(self, set_headers: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set_headers.0.remove(key);
            set_headers
        }
    }

    // insert
    impl<'set> HeaderAction<'set> for &'static str {
        #[inline] fn perform(self, set_headers: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set_headers.0.insert(key, CowSlice::Ref(unsafe {Slice::from_bytes(self.as_bytes())}));
            set_headers
        }
    }
    impl<'set> HeaderAction<'set> for String {
        #[inline] fn perform(self, set_headers: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set_headers.0.insert(key, CowSlice::Own(self.into_bytes()));
            set_headers
        }
    }
    impl<'set> HeaderAction<'set> for std::borrow::Cow<'static, str> {
        fn perform(self, set_headers: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set_headers.0.insert(key, CowSlice::Ref(unsafe {Slice::from_bytes(self.as_bytes())}));
            set_headers
        }
    }

    // append
    impl<'set, F: FnMut(&mut Value)> HeaderAction<'set> for F {
        fn perform(mut self, set_headers: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            self(unsafe {set_headers.0.values.get_unchecked_mut(key as usize)});
            set_headers
        }
    }
};

macro_rules! Header {
    ($N:literal; $( $konst:ident: $name_bytes:literal | $lower_case:literal $(| $other_pattern:literal)* , )*) => {
        pub(crate) const N_CLIENT_HEADERS: usize = $N;
        pub(crate) const CLIENT_HEADERS: [Header; N_CLIENT_HEADERS] = [ $( Header::$konst ),* ];

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
                pub fn $konst(self, action: impl HeaderAction<'set>) -> Self {
                    action.perform(self, Header::$konst)
                }
            )*
        }
        #[allow(non_snake_case)]
        impl Headers {
            $(
                pub fn $konst(&self) -> Option<&str> {
                    self.get(Header::$konst)
                }
            )*
        }

        // =================================================

        #[cfg(test)]
        #[test] fn client_header_name_cases() {
            $(
                assert_eq!(
                    std::str::from_utf8(&$name_bytes.to_ascii_lowercase()).unwrap(),
                    std::str::from_utf8($lower_case).unwrap(),
                );
            )*
        }
    };
} Header! {43;
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
    XRequestID:                  b"X-Request-ID" | b"x-request-id" | b"X-Request-Id",
}


impl Headers {
    #[inline] pub(crate) fn insert(&mut self, name: Header, value: CowSlice) {
        unsafe {*self.values.get_unchecked_mut(name as usize) = Value(Some(value))}
    }

    pub(crate) fn remove(&mut self, name: Header) {
        unsafe {*self.values.get_unchecked_mut(name as usize) = Value(None)}
    }

    #[inline] pub(crate) fn get(&self, name: Header) -> Option<&str> {
        match unsafe {&self.values.get_unchecked(name as usize).0} {
            Some(v) => Some(std::str::from_utf8(
                unsafe {v.as_bytes()}
            ).expect("Header value is not UTF-8")),
            None => None,
        }
    }
}
impl Headers {
    pub(crate) const fn init() -> Self {
        Self { values: [
            Value(None), Value(None), Value(None), Value(None), Value(None),
            Value(None), Value(None), Value(None), Value(None), Value(None),
            Value(None), Value(None), Value(None), Value(None), Value(None),
            Value(None), Value(None), Value(None), Value(None), Value(None),
            Value(None), Value(None), Value(None), Value(None), Value(None),
            Value(None), Value(None), Value(None), Value(None), Value(None),
            Value(None), Value(None), Value(None), Value(None), Value(None),
            Value(None), Value(None), Value(None), Value(None), Value(None),
            Value(None), Value(None), Value(None),
        ] }
    }
    #[cfg(test)] pub(crate) fn from_iter(iter: impl IntoIterator<Item = (Header, &'static str)>) -> Self {
        let mut this = Self::init();
        for (k, v) in iter {
            this.insert(k, CowSlice::Own(v.as_bytes().to_vec()))
        }
        this
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        struct Iter<'i> {
            map: &'i Headers,
            cur: usize,
        }
        impl<'i> Iterator for Iter<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                for i in self.cur..N_CLIENT_HEADERS {
                    if let Some(v) = unsafe {&self.map.values.get_unchecked(i).0} {
                        self.cur = i + 1;
                        return Some((
                            unsafe {CLIENT_HEADERS.get_unchecked(i)}.as_str(),
                            std::str::from_utf8(unsafe {v.as_bytes()}).expect("Header value is not UTF-8"),
                        ))
                    }
                }
                None
            }
        }

        Iter { map: self, cur: 0 }
    }
}
const _: () = {
    impl PartialEq for Headers {
        fn eq(&self, other: &Self) -> bool {
            for (k, v) in self.iter() {
                if other.get(Header::from_bytes(k.as_bytes()).unwrap()) != Some(v) {
                    return false
                }
            }
            true
        }
    }
};
