use std::borrow::Cow;


pub struct Headers {
    values: [Value; N_SERVER_HEADERS],

    /// Size of whole the byte stream when this is written into HTTP response.
    size: usize,
}
pub struct Value(
    Option<Cow<'static, [u8]>>,
);

pub trait HeaderAction<'headers> {
    type Output;
    fn perform(self, headers: &'headers mut Headers, key: Header) -> Self::Output;
} const _: () = {
    // get
    impl<'h> HeaderAction<'h> for () {
        type Output = Option<&'h [u8]>;
        fn perform(self, headers: &mut Headers, key: Header) -> Self::Output {
            headers.get(key)
        }
    }

    // remove
    impl<'h> HeaderAction<'h> for Option<()> {
        type Output = &'h mut Headers;
        fn perform(self, headers: &'h mut Headers, key: Header) -> Self::Output {
            headers.remove(key);
            headers
        }
    }

    // insert
    impl<'h> HeaderAction<'h> for &'static str {
        type Output = &'h mut Headers;
        fn perform(self, headers: &'h mut Headers, key: Header) -> Self::Output {
            headers.insert(key, Cow::Borrowed(self.as_bytes()));
            headers
        }
    }
    impl<'h> HeaderAction<'h> for String {
        type Output = &'h mut Headers;
        fn perform(self, headers: &'h mut Headers, key: Header) -> Self::Output {
            headers.insert(key, Cow::Owned(self.into_bytes()));
            headers
        }
    }

    // append
    impl<'h, F: FnMut(&mut Value)> HeaderAction<'h> for F {
        type Output = &'h mut Headers;
        fn perform(mut self, headers: &'h mut Headers, key: Header) -> Self::Output {
            self(&mut headers.values[key as usize]);
            headers
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
            #[inline] pub fn as_bytes(&self) -> &'static [u8] {
                match self {
                    $(
                        Self::$konst => $name_bytes,
                    )*
                }
            }
            #[inline] pub fn as_str(&self) -> &'static str {
                unsafe {std::str::from_utf8_unchecked(self.as_bytes())}
            }

            #[cfg(test)] pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
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
                pub fn $konst<'a, Action:HeaderAction<'a>>(&mut self, action: Action) -> Action::Output {
                    action.perform(self, Header::$konst)
                }
            )*
        }
    };
} Header! {47;
    AcceptRanges:                    b"Accept-Ranges",
    AccessControlAllowCredentials:   b"Access-Control-Allow-Credentials",
    AccessControlAllowHeaders:       b"Access-Control-Allow-Headers",
    AccessControlAllowMethods:       b"Access-Control-Allow-Methods",
    AccessControlAllowOrigin:        b"Access-Control-Allow-Origin",
    AccessControlExposeHeaders:      b"Access-Control-Expose-Headers",
    AccessControlMaxAge:             b"Access-Control-Max-Age",
    AccessControlRequestHeaders:     b"Access-Control-Request-Headers",
    AccessControlRequestMethod:      b"Access-Control-Request-Method",
    Age:                             b"Age",
    Allow:                           b"Allow",
    AltSrv:                          b"Alt-Srv",
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
    #[inline] pub(crate) fn insert(&mut self, name: Header, value: Cow<'static, [u8]>) {
        let (name_len, value_len) = (name.as_bytes().len(), value.len());
        match self.values[name as usize].0.replace(value) {
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

    pub(crate) fn append(&mut self, name: Header, value: Cow<'static, [u8]>) {
        let name_len = name.as_bytes().len();
        let index = name as usize;
        match &mut self.values[index] {
            Value(None) => {
                self.size += name_len + ": ".len() + value.len() + "\r\n".len();
                self.values[index].0 = Some(value);
            }
            Value(Some(v)) => {
                let before = v.len();
                let mut new = v.to_vec();
                new.extend_from_slice(&value);
                *v = Cow::Owned(new);
                self.size += v.len() - before;
            }
        }
    }

    #[inline] pub(crate) fn remove(&mut self, name: Header) {
        let name_len = name.as_bytes().len();
        let v = &mut self.values[name as usize];
        if let Some(v) = v.0.take() {
            self.size -= name_len + ": ".len() + v.len() + "\r\n".len()
        }
    }

    pub(crate) fn get(&self, name: Header) -> Option<&[u8]> {
        self.values[name as usize].0.as_ref().map(AsRef::as_ref)
    }
}
impl Headers {
    pub(crate) fn new() -> Self {
        Self {
            values: std::array::from_fn(|_| Value(None)),
            size:   "\r\n".len(),
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &[u8])> {
        struct Iter<'i> {
            map: &'i Headers,
            cur: usize,
        }
        impl<'i> Iterator for Iter<'i> {
            type Item = (&'i str, &'i [u8]);
            fn next(&mut self) -> Option<Self::Item> {
                for i in self.cur..N_SERVER_HEADERS {
                    if let Value(Some(v)) = &self.map.values[i] {
                        self.cur = i + 1;
                        return Some((SERVER_HEADERS[i].as_str(), &v))
                    }
                }
                None
            }
        }

        Iter { map: self, cur: 0 }
    }

    pub(crate) fn write_to(self, buf: &mut Vec<u8>) {
        macro_rules! write {
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

        for h in &SERVER_HEADERS {
            if let Some(v) = &self.values[*h as usize].0 {
                write!(buf <- h.as_bytes());
                write!(buf <- b": ");
                write!(buf <- v);
                write!(buf <- b"\r\n");
            }
        }
        write!(buf <- b"\r\n");
    }
}

const _: () = {
    use std::fmt::Debug;

    impl Debug for Headers {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_map()
                .entries(self.iter().map(|(k, v)| (k, v.escape_ascii())))
                .finish()
        }
    }
};
