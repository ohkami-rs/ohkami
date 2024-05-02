use std::borrow::Cow;
use crate::header::private::Append;
use ohkami_lib::{CowSlice, Slice};
use rustc_hash::FxHashMap;


#[derive(PartialEq)]
pub struct Headers {
    standard: Box<[Option<CowSlice>; N_CLIENT_HEADERS]>,
    custom:   Option<Box<FxHashMap<Slice, CowSlice>>>,
}

pub struct SetHeaders<'set>(
    &'set mut Headers
); impl Headers {
    pub fn set(&mut self) -> SetHeaders<'_> {
        SetHeaders(self)
    }
}

pub trait HeaderAction<'set> {
    fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set>;
} const _: () = {
    // remove
    impl<'set> HeaderAction<'set> for Option<()> {
        fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set.0.remove(key);
            set
        }
    }

    // append
    impl<'set> HeaderAction<'set> for Append {
        fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set.0.append(key, self.0.into());
            set
        }
    }

    // insert
    impl<'set> HeaderAction<'set> for &'static str {
        #[inline] fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set.0.insert(key, CowSlice::Ref(Slice::from_bytes(self.as_bytes())));
            set
        }
    }
    impl<'set> HeaderAction<'set> for String {
        #[inline] fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set.0.insert(key, CowSlice::Own(self.into_bytes().into_boxed_slice()));
            set
        }
    }
    impl<'set> HeaderAction<'set> for std::borrow::Cow<'static, str> {
        fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set.0.insert(key, CowSlice::Ref(Slice::from_bytes(self.as_bytes())));
            set
        }
    }
};

pub trait CustomHeadersAction<'set> {
    fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set>;
}
const _: () = {
    // remove
    impl<'set> CustomHeadersAction<'set> for Option<()> {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            if let Some(c) = &mut set.0.custom {
                c.remove(&Slice::from_bytes(key.as_bytes()));
            }
            set
        }
    }

    // append
    impl<'set> CustomHeadersAction<'set> for Append {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            let key = Slice::from_bytes(key.as_bytes());
            let custom = set.0.get_or_init_custom_mut();

            match custom.get_mut(&key) {
                Some(v) => unsafe {
                    v.extend_from_slice(b",");
                    v.extend_from_slice(self.0.as_bytes());
                }
                None => {
                    custom.insert(key, CowSlice::from(self.0));
                }
            }

            set
        }
    }

    // insert
    impl<'set> CustomHeadersAction<'set> for &'static str {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set.0.get_or_init_custom_mut().insert(
                Slice::from_bytes(key.as_bytes()),
                CowSlice::Ref(Slice::from_bytes(self.as_bytes()))
            );
            set
        }
    }
    impl<'set> CustomHeadersAction<'set> for String {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set.0.get_or_init_custom_mut().insert(
                Slice::from_bytes(key.as_bytes()),
                CowSlice::Own(self.into_bytes().into_boxed_slice())
            );
            set
        }
    }
    impl<'set> CustomHeadersAction<'set> for Cow<'static, str> {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set.0.get_or_init_custom_mut().insert(
                Slice::from_bytes(key.as_bytes()),
                CowSlice::from(self)
            );
            set
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
            #[inline]
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(
                        Self::$konst => unsafe {std::str::from_utf8_unchecked($name_bytes)},
                    )*
                }
            }
            #[inline(always)]
            pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
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
                #[inline(always)]
                pub fn $konst(self, action: impl HeaderAction<'set>) -> Self {
                    action.perform(self, Header::$konst)
                }
            )*

            pub fn custom(self, name: &'static str, action: impl CustomHeadersAction<'set>) -> Self {
                if self.0.custom.is_none() {
                    self.0.custom = Some(Box::new(FxHashMap::default()));
                }
                action.perform(self, name)
            }
        }

        #[allow(non_snake_case)]
        impl Headers {
            $(
                /// See the header value(s).
                /// 
                /// Multiple values are conbined into a comma-separated string, that can be iterated just by `.split(", ")`,\
                /// except for `Cookie` that by semicolon (see `Cookies` helper method).
                #[inline(always)]
                pub fn $konst(&self) -> Option<&str> {
                    self.get(Header::$konst)
                }
            )*

            pub fn custom(&self, name: &str) -> Option<&str> {
                let value = self.custom.as_ref()?
                    .get(&Slice::from_bytes(name.as_bytes()))?;
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

#[allow(non_snake_case)]
impl Headers {
    /// Util method to parse semicolon-separated Cookies into an iterator of
    /// `(name, value)`.
    /// 
    /// Invalid Cookie that doesn't contain `=` or contains multiple `=`s is just ignored.
    pub fn Cookies(&self) -> impl Iterator<Item = (&str, &str)> {
        self.Cookie()
            .map(|cookie_list| cookie_list.split("; ")
                .filter_map(|key_value| {
                    let mut key_value = key_value.split('=');
                    let key   = key_value.next()?;
                    let value = key_value.next()?;
                    key_value.next().is_none().then_some((key, value))
                })
            ).into_iter().flatten()
    }
}

impl Headers {
    #[inline(always)]
    fn get_or_init_custom_mut(&mut self) -> &mut FxHashMap<Slice, CowSlice> {
        self.custom.is_none().then(|| self.custom = Some(Box::new(FxHashMap::default())));
        unsafe {self.custom.as_mut().unwrap_unchecked()}
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

        struct Custom<'i> {
            map_iter: Option<::std::collections::hash_map::Iter<'i, Slice, CowSlice>>
        }
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

        Standard { cur:0, standard:&self.standard }
            .chain(Custom { map_iter: self.custom.as_ref().map(|box_hmap| box_hmap.iter()) })
            .chain(self.Cookie().map(|c| ("Cookie", c)))
    }
}

impl Headers {
    #[inline(always)] pub(crate) fn insert(&mut self, name: Header, value: CowSlice) {
        unsafe {*self.standard.get_unchecked_mut(name as usize) = Some(value)}
    }
    #[cfg(feature="DEBUG")]
    #[inline(always)] pub fn _insert(&mut self, name: Header, value: CowSlice) {
        unsafe {*self.standard.get_unchecked_mut(name as usize) = Some(value)}
    }

    pub(crate) fn remove(&mut self, name: Header) {
        unsafe {*self.standard.get_unchecked_mut(name as usize) = None}
    }

    #[inline] pub(crate) fn get(&self, name: Header) -> Option<&str> {
        match unsafe {self.standard.get_unchecked(name as usize)} {
            Some(v) => Some(std::str::from_utf8(
                unsafe {v.as_bytes()}
            ).expect("Header value is not UTF-8")),
            None => None,
        }
    }

    #[inline]
    pub(crate) fn append(&mut self, name: Header, value: CowSlice) {
        let target = unsafe {self.standard.get_unchecked_mut(name as usize)};

        match target {
            None => *target = Some(value),
            Some(v) => unsafe {
                v.extend_from_slice(b",");
                v.extend_from_slice(value.as_bytes());
            }
        }
    }
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
impl Headers {
    #[allow(unused)]
    #[inline] pub(crate) fn get_raw(&self, name: Header) -> Option<&CowSlice> {
        unsafe {self.standard.get_unchecked(name as usize)}.as_ref()
    }

    #[allow(unused)]
    #[inline] pub(crate) fn insert_custom(&mut self, name: Slice, value: CowSlice) {
        match &mut self.custom {
            Some(c) => {c.insert(name, value);}
            None => self.custom = Some(Box::new(FxHashMap::from_iter([
                (name, value)
            ])))
        }
    }
    #[cfg(feature="DEBUG")]
    #[inline] pub fn _insert_custom(&mut self, name: Slice, value: CowSlice) {
        self.insert_custom(name, value)
    }

    #[allow(unused)]
    #[inline] pub(crate) fn append_custom(&mut self, name: Slice, value: CowSlice) {
        let custom = self.get_or_init_custom_mut();

        match custom.get_mut(&name) {
            Some(v) => unsafe {
                v.extend_from_slice(b",");
                v.extend_from_slice(value.as_bytes());
            }
            None => {
                custom.insert(name, value);
            }
        }
    }
}

#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
impl Headers {
    #[inline]
    pub(crate) fn init() -> Self {
        Self {
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
            custom: None,
        }
    }
    #[cfg(feature="DEBUG")]
    pub fn _init() -> Self {
        Self::init()
    }

    #[allow(unused)]
    #[cfg(test)] pub(crate) fn from_iters(
        iter:   impl IntoIterator<Item = (Header, &'static str)>,
        custom: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> Self {
        let mut this = Self::init();
        for (k, v) in iter {
            this.insert(k, CowSlice::Ref(Slice::from_bytes(v.as_bytes())))
        }
        for (k, v) in custom {
            this.insert_custom(
                Slice::from_bytes(k.as_bytes()),
                CowSlice::Ref(Slice::from_bytes(v.as_bytes()))
            );
        }
        this
    }
}

#[cfg(feature="rt_worker")]
impl Headers {
    pub(crate) fn take_over(&mut self, w: &::worker::Headers) {
        for (k, v) in w.entries() {
            match Header::from_bytes(k.as_bytes()) {
                Some(standard) => self.insert(
                    standard,
                    CowSlice::Own(v.into_boxed_str().into())
                ),
                None => match &*k {
                    "Cookie" | "cookie" => self.append_cookie(
                        Slice::from_bytes(Box::leak(v.into_boxed_str()).as_bytes())
                    ),
                    _ => self.insert_custom(
                        Slice::from_bytes(Box::leak(k.into_boxed_str()).as_bytes()),
                        CowSlice::Own(v.into_boxed_str().into())
                    )
                }
            }
        }
    }
}

const _: () = {
    impl std::fmt::Debug for Headers {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_map().entries(self.iter()).finish()
        }
    }
};
