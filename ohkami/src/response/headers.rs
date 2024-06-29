use crate::header::private::{Append, SetCookie, SetCookieBuilder};
use std::borrow::Cow;
use rustc_hash::FxHashMap;


#[derive(Clone)]
pub struct Headers {
    standard:  Box<[Option<Cow<'static, str>>; N_SERVER_HEADERS]>,
    insertlog: Vec<usize>,
    custom:    Option<Box<FxHashMap<&'static str, Cow<'static, str>>>>,
    setcookie: Option<Box<Vec<Cow<'static, str>>>>,
    pub(crate) size: usize,
}

pub struct SetHeaders<'set>(
    &'set mut Headers
);
impl Headers {
    #[inline] pub fn set(&mut self) -> SetHeaders<'_> {
        SetHeaders(self)
    }
}

pub trait HeaderAction<'action> {
    fn perform(self, set: SetHeaders<'action>, key: Header) -> SetHeaders<'action>;
} const _: () = {
    // remove
    impl<'a> HeaderAction<'a> for Option<()> {
        #[inline] fn perform(self, set: SetHeaders<'a>, key: Header) -> SetHeaders<'a> {
            set.0.remove(key);
            set
        }
    }

    // append
    impl<'a> HeaderAction<'a> for Append {
        #[inline] fn perform(self, set: SetHeaders<'a>, key: Header) -> SetHeaders<'a> {
            set.0.append(key, self.0);
            set
        }
    }

    // insert
    impl<'a> HeaderAction<'a> for &'static str {
        #[inline(always)] fn perform(self, set: SetHeaders<'a>, key: Header) -> SetHeaders<'a> {
            set.0.insert(key, Cow::Borrowed(self));
            set
        }
    }
    impl<'a> HeaderAction<'a> for String {
        #[inline(always)] fn perform(self, set: SetHeaders<'a>, key: Header) -> SetHeaders<'a> {
            set.0.insert(key, Cow::Owned(self));
            set
        }
    }
    impl<'a> HeaderAction<'a> for std::borrow::Cow<'static, str> {
        fn perform(self, set: SetHeaders<'a>, key: Header) -> SetHeaders<'a> {
            set.0.insert(key, self);
            set
        }
    }
};

pub trait CustomHeadersAction<'action> {
    fn perform(self, set: SetHeaders<'action>, key: &'static str) -> SetHeaders<'action>;
}
const _: () = {
    /* remove */
    impl<'set> CustomHeadersAction<'set> for Option<()> {
        #[inline]
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            set.0.remove_custom(key);
            set
        }
    }

    /* append */
    impl<'set> CustomHeadersAction<'set> for Append {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            let self_len = self.0.len();

            let custom = {
                if set.0.custom.is_none() {
                    set.0.custom = Some(Box::new(FxHashMap::default()));
                }
                unsafe {set.0.custom.as_mut().unwrap_unchecked()}
            };

            set.0.size += match custom.get_mut(&key) {
                Some(value) => {
                    match value {
                        Cow::Owned(string) => {
                            string.push_str(", ");
                            string.push_str(&self.0);
                        }
                        Cow::Borrowed(s) => {
                            let mut s = s.to_string();
                            s.push_str(", ");
                            s.push_str(&self.0);
                            *value = Cow::Owned(s);
                        }
                    }
                    ", ".len() + self_len
                }
                None => {
                    custom.insert(key, self.0);
                    key.len() + ": ".len() + self_len + "\r\n".len()
                }
            };

            set
        }
    }

    /* insert */
    // specialize for `&'static str`:
    // NOT perform `let` binding of `self.len()`, using inlined `self.len()` instead.
    impl<'set> CustomHeadersAction<'set> for &'static str {
        #[inline(always)] fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            match &mut set.0.custom {
                None => {
                    set.0.custom = Some(Box::new(FxHashMap::from_iter([(key, Cow::Borrowed(self))])));
                    set.0.size += key.len() + ": ".len() + self.len() + "\r\n".len()
                }
                Some(custom) => {
                    if let Some(old) = custom.insert(key, Cow::Borrowed(self)) {
                        set.0.size -= old.len();
                        set.0.size += self.len();
                    } else {
                        set.0.size += key.len() + ": ".len() + self.len() + "\r\n".len()
                    }
                }
            }
            set
        }
    }
    impl<'set> CustomHeadersAction<'set> for String {
        #[inline(always)] fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            let self_len = self.len();
            match &mut set.0.custom {
                None => {
                    set.0.custom = Some(Box::new(FxHashMap::from_iter([(key, Cow::Owned(self))])));
                    set.0.size += key.len() + ": ".len() + self_len + "\r\n".len()
                }
                Some(custom) => {
                    if let Some(old) = custom.insert(key, Cow::Owned(self)) {
                        set.0.size -= old.len();
                        set.0.size += self_len;
                    } else {
                        set.0.size += key.len() + ": ".len() + self_len + "\r\n".len()
                    }
                }
            }
            set
        }
    }
    impl<'set> CustomHeadersAction<'set> for Cow<'static, str> {
        fn perform(self, set: SetHeaders<'set>, key: &'static str) -> SetHeaders<'set> {
            let self_len = self.len();
            match &mut set.0.custom {
                None => {
                    set.0.custom = Some(Box::new(FxHashMap::from_iter([(key, self)])));
                    set.0.size += key.len() + ": ".len() + self_len + "\r\n".len()
                }
                Some(custom) => {
                    if let Some(old) = custom.insert(key, self) {
                        set.0.size -= old.len();
                        set.0.size += self_len;
                    } else {
                        set.0.size += key.len() + ": ".len() + self_len + "\r\n".len()
                    }
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
            pub const fn as_str(&self) -> &'static str {
                unsafe {std::str::from_utf8_unchecked(self.as_bytes())}
            }
            #[inline(always)] const fn len(&self) -> usize {
                match self {
                    $(
                        Self::$konst => $name_bytes.len(),
                    )*
                }
            }

            // Mainly used in tests
            pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
                SERVER_HEADERS.into_iter()
                    .find(|h| h.as_bytes().eq_ignore_ascii_case(bytes))
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
                #[inline]
                pub fn $konst(self, action: impl HeaderAction<'set>) -> Self {
                    action.perform(self, Header::$konst)
                }
            )*

            #[inline]
            pub fn custom(self, name: &'static str, action: impl CustomHeadersAction<'set>) -> Self {
                action.perform(self, name)
            }
        }

        #[allow(non_snake_case)]
        impl Headers {
            $(
                #[inline]
                pub fn $konst(&self) -> Option<&str> {
                    self.get(Header::$konst)
                }
            )*

            #[inline]
            pub fn custom(&self, name: &'static str) -> Option<&str> {
                self.get_custom(name)
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
    StrictTransportSecurity:         b"Strict-Transport-Security",
    Trailer:                         b"Trailer",
    TransferEncoding:                b"Transfer-Encoding",
    Upgrade:                         b"Upgrade",
    Vary:                            b"Vary",
    Via:                             b"Via",
    WWWAuthenticate:                 b"WWW-Authenticate",
    XContentTypeOptions:             b"X-Content-Type-Options",
    XFrameOptions:                   b"X-Frame-Options",
}

const _: () = {
    #[allow(non_snake_case)]
    impl Headers {
        pub fn SetCookie(&self) -> impl Iterator<Item = SetCookie<'_>> {
            self.setcookie.as_ref().map(|setcookies|
                setcookies.iter().filter_map(|raw| match SetCookie::from_raw(raw) {
                    Ok(valid) => Some(valid),
                    Err(_err) => {
                        #[cfg(debug_assertions)] crate::warning!(
                            "Invalid `Set-Cookie`: {_err}"
                        );
                        None
                    }
                })
            ).into_iter().flatten()
        }
    }

    #[allow(non_snake_case)]
    impl<'s> SetHeaders<'s> {
        /// Add new `Set-Cookie` header in the response.
        /// 
        /// - When you call this N times, the response has N different
        ///   `Set-Cookie` headers.
        /// - Cookie value (second argument) is precent encoded when the
        ///   response is sended.
        /// 
        /// ---
        /// *example.rs*
        /// ```
        /// use ohkami::Response;
        /// 
        /// fn mutate_header(res: &mut Response) {
        ///     res.headers.set()
        ///         .Server("ohkami")
        ///         .SetCookie("id", "42", |d|d.Path("/").SameSiteLax())
        ///         .SetCookie("name", "John", |d|d.Path("/where").SameSiteStrict());
        /// }
        /// ```
        #[inline]
        pub fn SetCookie(self,
            name:  &'static str,
            value: impl Into<Cow<'static, str>>,
            directives: impl FnOnce(SetCookieBuilder)->SetCookieBuilder
        ) -> Self {
            let setcookie: Cow<'static, str> = directives(SetCookieBuilder::new(name, value)).build().into();
            self.0.size += "Set-Cookie: ".len() + setcookie.len() + "\r\n".len();
            match self.0.setcookie.as_mut() {
                None             => self.0.setcookie = Some(Box::new(vec![setcookie])),
                Some(setcookies) => setcookies.push(setcookie),
            }
            self
        }
    }
};

impl Headers {
    #[inline(always)]
    pub(crate) fn insert(&mut self, name: Header, value: Cow<'static, str>) {
        let (name_len, value_len) = (name.len(), value.len());
        match unsafe {self.standard.get_unchecked_mut(name as usize)}.replace(value) {
            None => {
                self.size += name_len + ": ".len() + value_len + "\r\n".len();
                self.insertlog.push(name as usize)
            }
            Some(old) => {self.size -= old.len(); self.size += value_len}
        }
    }

    #[inline]
    pub(crate) fn remove(&mut self, name: Header) {
        let name_len = name.len();
        let v = unsafe {self.standard.get_unchecked_mut(name as usize)};
        if let Some(v) = v.take() {
            self.size -= name_len + ": ".len() + v.len() + "\r\n".len()
        }
    }
    pub(crate) fn remove_custom(&mut self, name: &'static str) {
        if let Some(c) = self.custom.as_mut() {
            if let Some(v) = c.remove(name) {
                self.size -= name.len() + ": ".len() + v.len() + "\r\n".len()
            }
        }
    }

    #[inline(always)]
    pub(crate) fn get(&self, name: Header) -> Option<&str> {
        unsafe {self.standard.get_unchecked(name as usize)}.as_ref().map(AsRef::as_ref)
    }
    #[inline]
    pub(crate) fn get_custom(&self, name: &'static str) -> Option<&str> {
        self.custom.as_ref()?
            .get(name)
            .map(Cow::as_ref)
    }

    pub(crate) fn append(&mut self, name: Header, value: Cow<'static, str>) {
        let value_len = value.len();
        let target = unsafe {self.standard.get_unchecked_mut(name as usize)};

        self.size += match target {
            Some(v) => {
                match v {
                    Cow::Borrowed(slice) => {
                        let mut appended = String::with_capacity(slice.len() + 2 + value_len);
                        appended.push_str(slice);
                        appended.push_str(", ");
                        appended.push_str(&value);
                        *v = Cow::Owned(appended);
                    }
                    Cow::Owned(string) => {
                        string.push_str(", ");
                        string.push_str(&value);
                    }
                }
                ", ".len() + value_len
            }
            None => {
                *target = Some(value);
                self.insertlog.push(name as usize);
                name.len() + ": ".len() + value_len + "\r\n".len()
            }
        };
    }
}

impl Headers {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            standard:  Box::new([const {None}; N_SERVER_HEADERS]),
            insertlog: Vec::with_capacity(8),
            custom:    None,
            setcookie: None,
            size:      "\r\n".len(),
        }
    }
    #[cfg(feature="DEBUG")]
    #[doc(hidden)]
    pub fn _new() -> Self {Self::new()}

    pub(crate) const fn iter_standard(&self) -> impl Iterator<Item = (&str, &str)> {
        struct Standard<'i> {
            map: &'i Headers,
            cur: usize,
        }
        impl<'i> Iterator for Standard<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                for i in self.cur..N_SERVER_HEADERS {
                    if let Some(v) = unsafe {self.map.standard.get_unchecked(i)} {
                        self.cur = i + 1;
                        return Some((unsafe {SERVER_HEADERS.get_unchecked(i)}.as_str(), &v))
                    }
                }
                None
            }
        }

        Standard { map: self, cur: 0 }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        struct Standard<'i> {
            map: &'i [Option<Cow<'static, str>>; N_SERVER_HEADERS],
            cur: usize,
        }
        impl<'i> Iterator for Standard<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                for i in self.cur..N_SERVER_HEADERS {
                    if let Some(v) = unsafe {self.map.get_unchecked(i)} {
                        self.cur = i + 1;
                        return Some((unsafe {SERVER_HEADERS.get_unchecked(i)}.as_str(), &v))
                    }
                }
                None
            }
        }

        struct Custom<'i> {
            map: Option<std::collections::hash_map::Iter<'i, &'static str, Cow<'static, str>>>,
        }
        impl<'i> Iterator for Custom<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                self.map.as_mut()?
                    .next().map(|(k, v)| (&**k, &**v))
            }
        }

        let setcookies = self.setcookie.as_ref().map(|setcookies|
            setcookies.iter().map(Cow::as_ref)
        ).into_iter().flatten();

        Standard { map: &self.standard, cur: 0 }
            .chain(Custom { map: self.custom.as_ref().map(|box_hmap| box_hmap.iter()) })
            .chain(setcookies.map(|setcookie| ("Set-Cookie", setcookie)))
    }

    #[cfg(any(
        feature="rt_tokio",feature="rt_async-std",
        feature="DEBUG"
    ))]
    /// SAFETY: `buf` has remaining capacity of at least `self.size`
    pub(crate) unsafe fn write_unchecked_to(&self, buf: &mut Vec<u8>) {
        for i in &self.insertlog {
            if let Some(v) = unsafe {self.standard.get_unchecked(*i)} {
                crate::push_unchecked!(buf <- SERVER_HEADERS.get_unchecked(*i).as_bytes());
                crate::push_unchecked!(buf <- b": ");
                crate::push_unchecked!(buf <- v.as_bytes());
                crate::push_unchecked!(buf <- b"\r\n");
            }
        }
        if let Some(custom) = self.custom.as_ref() {
            for (k, v) in &**custom {
                crate::push_unchecked!(buf <- k.as_bytes());
                crate::push_unchecked!(buf <- b": ");
                crate::push_unchecked!(buf <- v.as_bytes());
                crate::push_unchecked!(buf <- b"\r\n");
            }
        }
        if let Some(setcookies) = self.setcookie.as_ref() {
            for setcookie in &**setcookies {
                crate::push_unchecked!(buf <- b"Set-Cookie: ");
                crate::push_unchecked!(buf <- setcookie.as_bytes());
                crate::push_unchecked!(buf <- b"\r\n");
            }
        }
        crate::push_unchecked!(buf <- b"\r\n");
    }

    #[cfg(feature="DEBUG")]
    pub fn _write_to(&self, buf: &mut Vec<u8>) {
        buf.reserve_exact(self.size);
        unsafe {self.write_unchecked_to(buf)}
    }
}

const _: () = {
    impl std::fmt::Debug for Headers {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_map()
                .entries(self.iter())
                .finish()
        }
    }

    impl PartialEq for Headers {
        fn eq(&self, other: &Self) -> bool {
            for (k, v) in self.iter_standard() {
                if other.get(Header::from_bytes(k.as_bytes()).unwrap()) != Some(v) {
                    return false
                }
            }

            if self.custom != other.custom {
                return false
            }

            true
        }
    }

    impl Headers {
        pub fn from_iter(iter: impl IntoIterator<Item = (
            &'static str,
            impl Into<Cow<'static, str>>)>
        ) -> Self {
            let mut this = Headers::new();
            for (k, v) in iter {
                match Header::from_bytes(k.as_bytes()) {
                    Some(h) => this.insert(h, v.into()),
                    None    => {this.set().custom(k, v.into());}
                }
            }
            this
        }
    }
};

#[cfg(feature="rt_worker")]
const _: () = {
    impl Into<::worker::Headers> for Headers {
        #[inline(always)]
        fn into(self) -> ::worker::Headers {
            let mut h = ::worker::Headers::new();
            for (k, v) in self.iter() {
                if let Err(_e) = h.append(k, v) {
                    #[cfg(feature="DEBUG")] println!("`worker::Headers::append` failed: {_e:?}");
                }
            }
            h
        }
    }
};
