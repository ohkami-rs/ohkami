use std::borrow::Cow;
use crate::layer0_lib::{Append, CowSlice, Slice};

#[cfg(feature="custom-header")]
use rustc_hash::FxHashMap;


pub struct Headers {
    values: [Option<CowSlice>; N_CLIENT_HEADERS],

    #[cfg(feature="custom-header")]
    custom: Option<Box<CustomHeaderMap>>,
}

#[cfg(feature="custom-header")]
type CustomHeaderMap = FxHashMap<CowSlice, CowSlice>;

pub struct SetHeaders<'set>(
    &'set mut Headers
); impl Headers {
    pub fn set(&mut self) -> SetHeaders<'_> {
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

    // append
    impl<'set> HeaderAction<'set> for Append {
        fn perform(self, set_headers: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set_headers.0.append(key, self.0);
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
};

#[cfg(feature="custom-header")]
pub trait CustomHeadersAction<'set> {
    fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set>;
}
#[cfg(feature="custom-header")]
const _: () = {
    // remove
    impl<'set> CustomHeadersAction<'set> for Option<()> {
        fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set> {
            if let Some(c) = &mut set_headers.0.custom {
                c.remove(&CowSlice::from(key.into()));
            }
            set_headers
        }
    }

    // append
    impl<'set> CustomHeadersAction<'set> for Append {
        fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set> {
            let key = CowSlice::from(key.into());

            if let Some(c) = &mut set_headers.0.custom {
                if let Some(value) = c.get_mut(&key) {
                    unsafe {value.extend(b",")}
                    unsafe {value.extend(self.0.as_bytes())}
                } else {
                    c.insert(key, CowSlice::from(self.0));
                }
            } else {
                set_headers.0.custom = Some(Box::new(CustomHeaderMap::from_iter([(
                    CowSlice::from(key),
                    CowSlice::from(self.0)
                )])))
            }

            set_headers
        }
    }

    // insert
    impl<'set> CustomHeadersAction<'set> for &'static str {
        fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set> {
            match &mut set_headers.0.custom {
                None => set_headers.0.custom = Some(Box::new(
                    CustomHeaderMap::from_iter([(
                        CowSlice::from(key.into()),
                        CowSlice::Ref(unsafe {Slice::from_bytes(self.as_bytes())})
                    )])
                )),
                Some(c) => {c.insert(
                    CowSlice::from(key.into()),
                    CowSlice::Ref(unsafe {Slice::from_bytes(self.as_bytes())})
                );}
            }
            set_headers
        }
    }
    impl<'set> CustomHeadersAction<'set> for String {
        fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set> {
            match &mut set_headers.0.custom {
                None => set_headers.0.custom = Some(Box::new(
                    CustomHeaderMap::from_iter([(
                        CowSlice::from(key.into()),
                        CowSlice::Own(self.into_bytes())
                    )])
                )),
                Some(c) => {c.insert(
                    CowSlice::from(key.into()),
                    CowSlice::Own(self.into_bytes())
                );}
            }
            set_headers
        }
    }
    impl<'set> CustomHeadersAction<'set> for Cow<'static, str> {
        fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set> {
            match &mut set_headers.0.custom {
                None => set_headers.0.custom = Some(Box::new(
                    CustomHeaderMap::from_iter([(
                        CowSlice::from(key.into()),
                        CowSlice::from(self)
                    )])
                )),
                Some(c) => {c.insert(
                    CowSlice::from(key.into()),
                    CowSlice::from(self)
                );}
            }
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

            #[cfg(feature="custom-header")]
            /// **`custom-header` feature required**
            pub fn custom(self, name: impl Into<Cow<'static, str>>, action: impl CustomHeadersAction<'set>) -> Self {
                if self.0.custom.is_none() {
                    self.0.custom = Some(Box::new(CustomHeaderMap::default()));
                }
                action.perform(self, name)
            }
        }

        #[allow(non_snake_case)]
        impl Headers {
            $(
                pub fn $konst(&self) -> Option<&str> {
                    self.get(Header::$konst)
                }
            )*

            #[cfg(feature="custom-header")]
            pub fn custom(&self, name: &str) -> Option<&str> {
                let value = self.custom.as_ref()?
                    .get(&CowSlice::Ref(unsafe {Slice::from_bytes(name.as_bytes())}))?;
                Some(std::str::from_utf8(unsafe {value.as_bytes()}).expect("Header value is not UTF-8"))
            }
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

impl Headers {
    #[inline(always)] pub(crate) fn insert(&mut self, name: Header, value: CowSlice) {
        unsafe {*self.values.get_unchecked_mut(name as usize) = Some(value)}
    }

    pub(crate) fn remove(&mut self, name: Header) {
        unsafe {*self.values.get_unchecked_mut(name as usize) = None}
    }

    #[inline] pub(crate) fn get(&self, name: Header) -> Option<&str> {
        match unsafe {self.values.get_unchecked(name as usize)} {
            Some(v) => Some(std::str::from_utf8(
                unsafe {v.as_bytes()}
            ).expect("Header value is not UTF-8")),
            None => None,
        }
    }

    pub(crate) fn append(&mut self, name: Header, value: Cow<'static, str>) {
        let value_len = value.len();
        let target = unsafe {self.values.get_unchecked_mut(name as usize)};

        match target {
            Some(v) => {
                match v {
                    CowSlice::Ref(slice) => {
                        let mut appended = unsafe {slice.as_bytes()}.to_vec();
                        appended.push(b',');
                        appended.extend_from_slice(value.as_bytes());
                    }
                    CowSlice::Own(vec) => {
                        vec.push(b',');
                        vec.extend_from_slice(value.as_bytes());
                    }
                }
                value_len + 1
            }
            None => {
                *target = Some(CowSlice::Ref(unsafe {Slice::from_bytes(value.as_bytes())}));
                value_len
            }
        };
    }
}

#[cfg(feature="custom-header")]
impl Headers {
    #[inline] pub(crate) fn insert_custom(&mut self, name: CowSlice, value: CowSlice) {
        match &mut self.custom {
            Some(c) => {c.insert(name, value);}
            None => self.custom = Some(Box::new(CustomHeaderMap::from_iter([
                (name, value)
            ])))
        }
    }
}

impl Headers {
    pub(crate) const fn init() -> Self {
        Self {
            values: [
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None,
            ],
            #[cfg(feature="custom-header")]
            custom: None,
        }
    }

    #[cfg(test)] pub(crate) fn from_iter(iter: impl IntoIterator<Item = (Header, &'static str)>) -> Self {
        let mut this = Self::init();
        for (k, v) in iter {
            this.insert(k, CowSlice::Own(v.as_bytes().to_vec()))
        }
        this
    }
    #[cfg(feature="custom-header")]
    #[cfg(test)] pub(crate) fn from_iters(
        iter:   impl IntoIterator<Item = (Header, &'static str)>,
        custom: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> Self {
        let mut this = Self::init();
        for (k, v) in iter {
            this.insert(k, CowSlice::Own(v.as_bytes().to_vec()))
        }
        for (k, v) in custom {
            this.insert_custom(
                CowSlice::Ref(unsafe {Slice::from_bytes(k.as_bytes())}),
                CowSlice::Ref(unsafe {Slice::from_bytes(v.as_bytes())})
            );
        }
        this
    }

    pub(crate) fn iter_standard(&self) -> impl Iterator<Item = (&str, &str)> {
        struct Standard<'i> {
            cur:      usize,
            standard: &'i [Option<CowSlice>; N_CLIENT_HEADERS],
        }
        impl<'i> Iterator for Standard<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                for i in self.cur..N_CLIENT_HEADERS {
                    if let Some(v) = unsafe {self.standard.get_unchecked(i)} {
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

        Standard { cur:0, standard:&self.values }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        struct Standard<'i> {
            cur:      usize,
            standard: &'i [Option<CowSlice>; N_CLIENT_HEADERS],
        }
        impl<'i> Iterator for Standard<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                for i in self.cur..N_CLIENT_HEADERS {
                    if let Some(v) = unsafe {self.standard.get_unchecked(i)} {
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

        #[cfg(feature="custom-header")]
        struct Custom<'i> {
            map_iter: Option<::std::collections::hash_map::Iter<'i, CowSlice, CowSlice>>
        }
        #[cfg(feature="custom-header")]
        impl<'i> Iterator for Custom<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                self.map_iter.as_mut()?
                    .next().map(|(k, v)| (
                        std::str::from_utf8(unsafe {k.as_bytes()}).expect("Header value is not UTF-8"),
                        std::str::from_utf8(unsafe {v.as_bytes()}).expect("Header value is not UTF-8")
                    ))
            }
        }

        #[cfg(feature="custom-header")] {
            Iterator::chain(
                Standard { cur:0, standard:&self.values },
                Custom { map_iter:self.custom.as_ref().map(|box_hmap| box_hmap.iter()) }
            )
        }
        #[cfg(not(feature="custom-header"))] {
            Standard { cur:0, standard:&self.values }
        }
    }
}

const _: () = {
    impl PartialEq for Headers {
        fn eq(&self, other: &Self) -> bool {
            for (k, v) in self.iter_standard() {
                if other.get(Header::from_bytes(k.as_bytes()).unwrap()) != Some(v) {
                    return false
                }
            }
            
            #[cfg(feature="custom-header")]
            if self.custom != other.custom {
                return false
            }

            true
        }
    }
};
