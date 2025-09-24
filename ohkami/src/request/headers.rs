use crate::header::{Append, ByteArrayMap};
use ohkami_lib::{CowSlice, Slice, map::TupleMap};
use std::borrow::Cow;

pub struct Headers {
    standard: ByteArrayMap<N_CLIENT_HEADERS, CowSlice>,
    custom: Option<Box<TupleMap<Slice, CowSlice>>>,
}

pub struct SetHeaders<'set>(&'set mut Headers);
impl Headers {
    #[inline]
    pub fn set(&mut self) -> SetHeaders<'_> {
        SetHeaders(self)
    }
}

pub trait HeaderAction<'set> {
    fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set>;
}
const _: () = {
    // append
    impl<'set> HeaderAction<'set> for Append {
        #[inline]
        fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set.0.append(key, self.0.into());
            set
        }
    }

    // insert
    impl<'set> HeaderAction<'set> for &'static str {
        #[inline]
        fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set.0
                .insert(key, CowSlice::Ref(Slice::from_bytes(self.as_bytes())));
            set
        }
    }
    impl<'set> HeaderAction<'set> for String {
        #[inline]
        fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set.0
                .insert(key, CowSlice::Own(self.into_bytes().into_boxed_slice()));
            set
        }
    }
    impl<'set> HeaderAction<'set> for std::borrow::Cow<'static, str> {
        fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            set.0
                .insert(key, CowSlice::Ref(Slice::from_bytes(self.as_bytes())));
            set
        }
    }

    // remove or insert
    impl<'set> HeaderAction<'set> for Option<Cow<'static, str>> {
        #[inline]
        fn perform(self, set: SetHeaders<'set>, key: Header) -> SetHeaders<'set> {
            match self {
                None => set.0.remove(key),
                Some(v) => set.0.insert(key, CowSlice::from(v)),
            }
            set
        }
    }
};

pub trait CustomHeadersAction<'set> {
    fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set>;
}
const _: () = {
    // append
    impl<'set> CustomHeadersAction<'set> for Append {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set.0
                .append_custom(Slice::from_bytes(key.as_bytes()), self.0.into());
            set
        }
    }

    // insert
    impl<'set> CustomHeadersAction<'set> for &'static str {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set.0.insert_custom(
                Slice::from_bytes(key.as_bytes()),
                CowSlice::Ref(Slice::from_bytes(self.as_bytes())),
            );
            set
        }
    }
    impl<'set> CustomHeadersAction<'set> for String {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set.0.insert_custom(
                Slice::from_bytes(key.as_bytes()),
                CowSlice::Own(self.into_bytes().into_boxed_slice()),
            );
            set
        }
    }
    impl<'set> CustomHeadersAction<'set> for Cow<'static, str> {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set.0
                .insert_custom(Slice::from_bytes(key.as_bytes()), CowSlice::from(self));
            set
        }
    }

    // remove or insert
    impl<'set> CustomHeadersAction<'set> for Option<Cow<'static, str>> {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            match self {
                None => {
                    if let Some(c) = &mut set.0.custom {
                        c.remove(Slice::from_bytes(key.as_bytes()));
                    }
                }
                Some(v) => set
                    .0
                    .insert_custom(Slice::from_bytes(key.as_bytes()), CowSlice::from(v)),
            }
            set
        }
    }
};

macro_rules! Header {
    ($N:literal; $( $method:ident($konst:ident): $name_bytes:literal | $lower_case:literal $(| $other_pattern:literal)* , )*) => {
        pub(crate) const N_CLIENT_HEADERS: usize = $N;
        const _: [Header; N_CLIENT_HEADERS] = [$(Header::$konst),*];

        #[derive(Debug, PartialEq, Clone, Copy)]
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
                pub fn $method(self, action: impl HeaderAction<'set>) -> Self {
                    action.perform(self, Header::$konst)
                }

                #[deprecated = "Use snake_case method instead"]
                pub fn $konst(self, action: impl HeaderAction<'set>) -> Self {
                    self.$method(action)
                }
            )*

            pub fn x(self, name: &'static str, action: impl CustomHeadersAction<'set>) -> Self {
                if self.0.custom.is_none() {
                    self.0.custom = Some(Box::new(TupleMap::new()));
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
                pub fn $method(&self) -> Option<&str> {
                    self.get_standard(Header::$konst)
                }

                #[deprecated = "Use snake_case method instead"]
                pub fn $konst(&self) -> Option<&str> {
                    self.$method()
                }
            )*

            pub fn get(&self, name: &str) -> Option<&str> {
                let value = self.custom.as_ref()?
                    .get(&Slice::from_bytes(name.as_bytes()))
                    .or_else(|| {
                        let standard = Header::from_bytes(name.as_bytes())?;
                        unsafe {self.standard.get(standard as u8)}
                    })?;
                std::str::from_utf8(unsafe {value.as_bytes()}).ok()
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
}
Header! {46;
    accept(Accept): b"Accept" | b"accept",
    accept_encoding(AcceptEncoding): b"Accept-Encoding" | b"accept-encoding",
    accept_language(AcceptLanguage): b"Accept-Language" | b"accept-language",
    access_control_request_headers(AccessControlRequestHeaders): b"Access-Control-Request-Headers" | b"access-control-request-headers",
    access_control_request_method(AccessControlRequestMethod): b"Access-Control-Request-Method" | b"access-control-request-method",
    authorization(Authorization): b"Authorization" | b"authorization",
    cache_control(CacheControl): b"Cache-Control" | b"cache-control",
    connection(Connection): b"Connection" | b"connection",
    content_disposition(ContentDisposition): b"Content-Disposition" | b"content-disposition",
    content_encoding(ContentEncoding): b"Content-Encoding" | b"content-encoding",
    content_laguage(ContentLanguage): b"Content-Language" | b"content-language",
    content_length(ContentLength): b"Content-Length" | b"content-length",
    content_location(ContentLocation): b"Content-Location" | b"content-location",
    content_type(ContentType): b"Content-Type" | b"content-type",
    cookie(Cookie): b"Cookie" | b"cookie",
    date(Date): b"Date" | b"date",
    expect(Expect): b"Expect" | b"expect",
    forwarded(Forwarded): b"Forwarded" | b"forwarded",
    from(From): b"From" | b"from",
    host(Host): b"Host" | b"host",
    if_match(IfMatch): b"If-Match" | b"if-match",
    if_modified_since(IfModifiedSince): b"If-Modified-Since" | b"if-modified-since",
    if_none_match(IfNoneMatch): b"If-None-Match" | b"if-none-match",
    if_range(IfRange): b"If-Range" | b"if-range",
    if_unmodified_since(IfUnmodifiedSince): b"If-Unmodified-Since" | b"if-unmodified-since",
    link(Link): b"Link" | b"link",
    max_forwards(MaxForwards): b"Max-Forwards" | b"max-forwards",
    origin(Origin): b"Origin" | b"origin",
    proxy_authorization(ProxyAuthorization): b"Proxy-Authorization" | b"proxy-authorization",
    range(Range): b"Range" | b"range",
    referer(Referer): b"Referer" | b"referer",
    sec_fetch_dest(SecFetchDest): b"Sec-Fetch-Dest" | b"sec-fetch-dest",
    sec_fetch_mode(SecFetchMode): b"Sec-Fetch-Mode" | b"sec-fetch-mode",
    sec_fetch_site(SecFetchSite): b"Sec-Fetch-Site" | b"sec-fetch-site",
    sec_fetch_user(SecFetchUser): b"Sec-Fetch-User" | b"sec-fetch-user",
    sec_websocket_extensions(SecWebSocketExtensions): b"Sec-WebSocket-Extensions" | b"sec-websocket-extensions",
    sec_websocket_key(SecWebSocketKey): b"Sec-WebSocket-Key" | b"sec-websocket-key",
    sec_websocket_protocol(SecWebSocketProtocol): b"Sec-WebSocket-Protocol" | b"sec-websocket-protocol",
    sec_websocket_version(SecWebSocketVersion): b"Sec-WebSocket-Version" | b"sec-websocket-version",
    te(TE): b"TE" | b"te",
    trailer(Trailer): b"Trailer" | b"trailer",
    transfer_encoding(TransferEncoding): b"Transfer-Encoding" | b"transfer-encoding",
    user_agent(UserAgent): b"User-Agent" | b"user-agent",
    upgrade(Upgrade): b"Upgrade" | b"upgrade",
    upgrade_insecure_requests(UpgradeInsecureRequests): b"Upgrade-Insecure-Requests" | b"upgrade-insecure-requests",
    via(Via): b"Via" | b"via",
}

#[allow(non_snake_case)]
impl Headers {
    /// Util method to parse semicolon-separated Cookies into an iterator of
    /// `(name, value)`.
    ///
    /// internally uses [`ohkami::util::iter_cookies`](crate::util::iter_cookies).
    pub fn Cookies(&self) -> impl Iterator<Item = (&str, &str)> {
        self.cookie()
            .map(crate::util::iter_cookies)
            .into_iter()
            .flatten()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.standard
            .iter()
            .filter_map(|(i, v)| {
                Some((
                    unsafe { std::mem::transmute::<_, Header>(*i).as_str() },
                    std::str::from_utf8(v).ok()?,
                ))
            })
            .chain(self.custom.as_ref().into_iter().flat_map(|hm| {
                hm.iter().filter_map(|(k, v)| {
                    Some((
                        std::str::from_utf8(unsafe { k.as_bytes() }).ok()?,
                        std::str::from_utf8(unsafe { v.as_bytes() }).ok()?,
                    ))
                })
            }))
            .chain(self.cookie().map(|c| ("Cookie", c)))
    }
}

impl Headers {
    #[inline(always)]
    pub(crate) fn insert(&mut self, name: Header, value: CowSlice) {
        unsafe { self.standard.insert(name as u8, value) }
    }
    #[cfg(feature = "DEBUG")]
    #[inline(always)]
    pub fn _insert(&mut self, name: Header, value: CowSlice) {
        self.insert(name, value)
    }

    pub(crate) fn remove(&mut self, name: Header) {
        unsafe { self.standard.delete(name as u8) };
    }

    #[inline]
    pub(crate) fn get_standard(&self, name: Header) -> Option<&str> {
        unsafe {
            match self.standard.get(name as u8) {
                Some(cs) => std::str::from_utf8(&cs).ok(),
                None => None,
            }
        }
    }

    #[inline(always)]
    pub(crate) fn append(&mut self, name: Header, value: CowSlice) {
        unsafe {
            match self.standard.get_mut(name as u8) {
                None => self.standard.insert_new(name as u8, value),
                Some(v) => {
                    v.extend_from_slice(b", ");
                    v.extend_from_slice(&value);
                }
            }
        }
    }
    #[cfg(feature = "DEBUG")]
    #[inline]
    pub fn _append(&mut self, name: Header, value: CowSlice) {
        self.append(name, value)
    }
}

impl Headers {
    #[inline]
    pub(crate) fn insert_custom(&mut self, name: Slice, value: CowSlice) {
        match &mut self.custom {
            Some(c) => {
                c.insert(name, value);
            }
            None => self.custom = Some(Box::new(TupleMap::from_iter([(name, value)]))),
        }
    }
    #[cfg(feature = "DEBUG")]
    #[inline]
    pub fn _insert_custom(&mut self, name: Slice, value: CowSlice) {
        self.insert_custom(name, value)
    }

    #[inline]
    pub(crate) fn append_custom(&mut self, name: Slice, value: CowSlice) {
        if self.custom.is_none() {
            self.custom = Some(Box::new(TupleMap::new()))
        }

        let c = unsafe { self.custom.as_mut().unwrap_unchecked() };

        match c.get_mut(&name) {
            Some(v) => unsafe {
                v.extend_from_slice(b", ");
                v.extend_from_slice(value.as_bytes());
            },
            None => {
                c.insert(name, value);
            }
        }
    }
}

#[cfg(feature = "__rt__")]
impl Headers {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            standard: ByteArrayMap::new(),
            custom: None,
        }
    }
    #[cfg(feature = "DEBUG")]
    pub fn _init() -> Self {
        Self::new()
    }

    #[cfg(feature = "__rt_native__")]
    #[inline]
    pub(crate) fn clear(&mut self) {
        self.standard.clear();
        if let Some(map) = &mut self.custom {
            map.clear()
        }
    }

    #[cfg(any(
        feature = "__rt_native__",
        all(debug_assertions, any(feature = "rt_worker", feature = "rt_lambda",))
    ))]
    #[inline]
    pub(crate) fn get_raw(&self, name: Header) -> Option<&CowSlice> {
        unsafe { self.standard.get(name as u8) }
    }

    #[allow(unused)]
    #[cfg(test)]
    pub(crate) fn from_iters(
        iter: impl IntoIterator<Item = (Header, &'static str)>,
        custom: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> Self {
        let mut this = Self::new();
        for (k, v) in iter {
            this.insert(k, CowSlice::Ref(Slice::from_bytes(v.as_bytes())))
        }
        for (k, v) in custom {
            this.insert_custom(
                Slice::from_bytes(k.as_bytes()),
                CowSlice::Ref(Slice::from_bytes(v.as_bytes())),
            );
        }
        this
    }
}

#[cfg(feature = "rt_worker")]
impl Headers {
    pub(crate) fn take_over(&mut self, w: &::worker::Headers) {
        for (k, v) in w.entries() {
            match Header::from_bytes(k.as_bytes()) {
                Some(standard) => self.insert(standard, CowSlice::Own(v.into_boxed_str().into())),
                None => self.insert_custom(
                    Slice::from_bytes(Box::leak(k.into_boxed_str()).as_bytes()),
                    CowSlice::Own(v.into_boxed_str().into()),
                ),
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

    impl PartialEq for Headers {
        fn eq(&self, other: &Self) -> bool {
            self.custom == other.custom && self.standard == other.standard
        }
    }
};
