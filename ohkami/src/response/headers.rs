use std::borrow::Cow;
use crate::layer0_lib::Append;

#[cfg(feature="custom-header")]
use rustc_hash::FxHashMap;


#[cfg(feature="custom-header")]
type CustomHeaderMap = FxHashMap<Cow<'static, str>, Cow<'static, str>>;

pub struct Headers {
    values: [Option<Cow<'static, str>>; N_SERVER_HEADERS],

    #[cfg(feature="custom-header")]
    custom: Option<Box<CustomHeaderMap>>,

    /// Size of whole the byte stream when this is written into HTTP response.
    size: usize,
}

pub struct SetHeaders<'set>(
    &'set mut Headers
); impl Headers {
    #[inline(always)] pub fn set(&mut self) -> SetHeaders<'_> {
        SetHeaders(self)
    }
}

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

    // append
    impl<'a> HeaderAction<'a> for Append {
        #[inline] fn perform(self, set_headers: SetHeaders<'a>, key: Header) -> SetHeaders<'a> {
            set_headers.0.append(key, self.0);
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

#[cfg(feature="custom-header")]
pub trait CustomHeadersAction<'action> {
    fn perform(self, set_headers: SetHeaders<'action>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'action>;
}
#[cfg(feature="custom-header")]
const _: () = {
    pub trait CustomHeadersAction<'set> {
        fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set>;
    }
    
    // remove
    impl<'set> CustomHeadersAction<'set> for Option<()> {
        fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set> {
            if let Some(c) = &mut set_headers.0.custom {
                if let Some(removed) = c.remove(&key.into()) {
                    set_headers.0.size -= removed.len();
                }

            }
            set_headers
        }
    }

    // append
    impl<'set> CustomHeadersAction<'set> for Append {
        fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set> {
            let self_len = self.0.len();
            let key = key.into();

            if let Some(c) = &mut set_headers.0.custom {
                if let Some(value) = c.get_mut(&key) {
                    match value {
                        Cow::Owned(string) => {string.push(','); string.push_str(&self.0);}
                        Cow::Borrowed(s) => {
                            let mut s = s.to_string();
                            s.push(','); s.push_str(&self.0);
                            *value = Cow::Owned(s);
                        }
                    }
                    set_headers.0.size += 1 + self_len;
                } else {
                    c.insert(key, self.0);
                    set_headers.0.size += self_len;
                }
            } else {
                set_headers.0.custom = Some(Box::new(CustomHeaderMap::from_iter([(
                    key,
                    self.0
                )])));
                set_headers.0.size += self_len;
            }

            set_headers
        }
    }

    // insert
    impl<'set> CustomHeadersAction<'set> for &'static str {
        #[inline] fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set> {
            match &mut set_headers.0.custom {
                None => set_headers.0.custom = Some(Box::new(CustomHeaderMap::from_iter([(key.into(), Cow::Borrowed(self))]))),
                Some(c) => {c.insert(key.into(), Cow::Borrowed(self));}
            }
            set_headers
        }
    }
    impl<'set> CustomHeadersAction<'set> for String {
        #[inline] fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set> {
            match &mut set_headers.0.custom {
                None => set_headers.0.custom = Some(Box::new(CustomHeaderMap::from_iter([(key.into(), Cow::Owned(self))]))),
                Some(c) => {c.insert(key.into(), Cow::Owned(self));}
            }
            set_headers
        }
    }
    impl<'set> CustomHeadersAction<'set> for Cow<'static, str> {
        fn perform(self, set_headers: SetHeaders<'set>, key: impl Into<Cow<'static, str>>) -> SetHeaders<'set> {
            match &mut set_headers.0.custom {
                None => set_headers.0.custom = Some(Box::new(CustomHeaderMap::from_iter([(key.into(), self)]))),
                Some(c) => {c.insert(key.into(), self);}
            }
            set_headers
        }
    }
};

//pub trait 

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
        impl Headers {
            $(
                pub fn $konst(&self) -> Option<&str> {
                    self.get(Header::$konst)
                }
            )*

            #[cfg(feature="custom-header")]
            pub fn custom(&self, name: impl Into<Cow<'static, str>>) -> Option<&str> {
                let value = self.custom.as_ref()?
                    .get(&name.into())?;
                Some(&value)
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
            pub fn custom(self, name: impl Into<Cow<'static, str>>, action: impl CustomHeadersAction<'set>) -> Self {
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

impl Headers {
    #[inline] pub(crate) fn insert(&mut self, name: Header, value: Cow<'static, str>) {
        let (name_len, value_len) = (name.as_bytes().len(), value.len());
        match unsafe {self.values.get_unchecked_mut(name as usize)}.replace(value) {
            None       => self.size += name_len + ": ".len() + value_len + "\r\n".len(),
            Some(prev) => {
                let prev_len = prev.len();
                if value_len > prev_len {
                    self.size += value_len - prev_len;
                } else {
                    self.size -= prev_len - value_len;
                }
            }
        }
    }

    #[inline] pub(crate) fn remove(&mut self, name: Header) {
        let name_len = name.as_bytes().len();
        let v = unsafe {self.values.get_unchecked_mut(name as usize)};
        if let Some(v) = v.take() {
            self.size -= name_len + ": ".len() + v.len() + "\r\n".len()
        }
    }

    pub(crate) fn get(&self, name: Header) -> Option<&str> {
        unsafe {self.values.get_unchecked(name as usize)}.as_ref().map(AsRef::as_ref)
    }

    pub(crate) fn append(&mut self, name: Header, value: Cow<'static, str>) {
        let value_len = value.len();
        let target = unsafe {self.values.get_unchecked_mut(name as usize)};

        let size_increase = match target {
            Some(v) => {
                match v {
                    Cow::Borrowed(slice) => {
                        let mut appended = String::with_capacity(slice.len() + 1 + value_len);
                        appended.push_str(slice);
                        appended.push(',');
                        appended.push_str(&value);
                    }
                    Cow::Owned(string) => {
                        string.push(',');
                        string.push_str(&value);
                    }
                }
                value_len + 1
            }
            None => {
                *target = Some(value);
                value_len
            }
        };
        self.size += size_increase;
    }
}
impl Headers {
    pub(crate) const fn new() -> Self {
        Self {
            size:   "\r\n".len(),
            values: [
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
                None, None, None, None, None,
            ],
            #[cfg(feature="custom-header")]
            custom: None
        }
    }
    #[cfg(feature="DEBUG")]
    #[doc(hidden)]
    pub const fn init() -> Self {Self::new()}

    pub(crate) const fn iter_standard(&self) -> impl Iterator<Item = (&str, &str)> {
        struct Standard<'i> {
            map: &'i Headers,
            cur: usize,
        }
        impl<'i> Iterator for Standard<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                for i in self.cur..N_SERVER_HEADERS {
                    if let Some(v) = unsafe {self.map.values.get_unchecked(i)} {
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

        #[cfg(feature="custom-header")]
        struct Custom<'i> {
            map: Option<std::collections::hash_map::Iter<'i, Cow<'static, str>, Cow<'static, str>>>,
        }
        #[cfg(feature="custom-header")]
        impl<'i> Iterator for Custom<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                self.map.as_mut()?
                    .next().map(|(k, v)| (&**k, &**v))
            }
        }

        #[cfg(feature="custom-header")] {
            Iterator::chain(
                Standard { map: &self.values, cur: 0 },
                Custom { map: self.custom.as_ref().map(|box_hmap| box_hmap.iter()) }
            )
        }
        #[cfg(not(feature="custom-header"))] {
            Standard { map: &self.values, cur: 0 }
        }
    }

    #[inline] pub(crate) fn write_to(self, buf: &mut Vec<u8>) {
        macro_rules! push {
            ($buf:ident <- $bytes:expr) => {
                unsafe {
                    let (buf_len, bytes_len) = ($buf.len(), $bytes.len());
                    std::ptr::copy_nonoverlapping(
                        $bytes.as_ptr(),
                        $buf.as_mut_ptr().add(buf_len),
                        bytes_len
                    );
                    $buf.set_len(buf_len + bytes_len);
                }
            };
        }

        buf.reserve(self.size);
        for h in unsafe {SERVER_HEADERS.get_unchecked(1..)} {
            if let Some(v) = unsafe {self.values.get_unchecked(*h as usize)} {
                push!(buf <- h.as_bytes());
                push!(buf <- b": ");
                push!(buf <- v);
                push!(buf <- b"\r\n");
            }
        }
        push!(buf <- b"\r\n");
    }
    #[cfg(feature="DEBUG")]
    #[doc(hidden)]
    #[inline] pub fn write_ref_to(&self, buf: &mut Vec<u8>) {
        macro_rules! push {
            ($buf:ident <- $bytes:expr) => {
                unsafe {
                    let (buf_len, bytes_len) = ($buf.len(), $bytes.len());
                    std::ptr::copy_nonoverlapping(
                        $bytes.as_ptr(),
                        $buf.as_mut_ptr().add(buf_len),
                        bytes_len
                    );
                    $buf.set_len(buf_len + bytes_len);
                }
            };
        }

        buf.reserve(self.size);
        for h in unsafe {SERVER_HEADERS.get_unchecked(1..)} {
            if let Some(v) = unsafe {self.values.get_unchecked(*h as usize)} {
                push!(buf <- h.as_bytes());
                push!(buf <- b": ");
                push!(buf <- v);
                push!(buf <- b"\r\n");
            }
        }
        push!(buf <- b"\r\n");
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

            #[cfg(feature="custom-header")]
            if self.custom != other.custom {
                return false
            }

            true
        }
    }

    impl Headers {
        pub fn from_iter(iter: impl IntoIterator<Item = (Header, impl Into<Cow<'static, str>>)>) -> Self {
            let mut this = Headers::new();
            for (k, v) in iter {
                this.insert(k, v.into())
            }
            this
        }
    }
};
