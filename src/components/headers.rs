use crate::utils::buffer::{BufRange, Buffer};


/// for request headers
pub(crate) struct HeaderMap(
    Vec<(BufRange, BufRange)>
); impl HeaderMap {
    pub fn get<'buf, K: HeaderKey>(&self, key: K, buffer: &'buf Buffer) -> Option<&'buf str> {
        let key = key.as_key_str();
        for (key_range, value_range) in &self.0 {
            if buffer.read_str(key_range) == key {
                return Some(buffer.read_str(value_range))
            }
        }
        None
    }

    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
    pub(crate) fn push(&mut self, key_range: BufRange, value_range: BufRange) {
        self.0.push((key_range, value_range))
    }

    pub(crate) fn debug_fmt_with(&self, buffer: &Buffer) -> String {
        self.0.iter().fold(
            String::new(),
            |it, (key_range, value_range)| {
                it + "\n" +
                buffer.read_str(key_range) +
                ": " +
                buffer.read_str(value_range)
            }
        )
    }
}

pub trait HeaderKey {fn as_key_str(self) -> &'static str;}
impl HeaderKey for &'static str {fn as_key_str(self) -> &'static str {self}}
impl HeaderKey for Header {fn as_key_str(self) -> &'static str {self.as_str()}}

pub enum Header {
    // request
    Accept,
    AcceptEncoding,
    AcceptLanguage,
    Authorization,
    ContentType,
    Expect,
    From,
    Host,
    IfMatch,
    IfModifiedSince,
    IfNoneMatch,
    IfRange,
    IfUnmodifiedSince,
    MaxForwords,
    ProxyAuthorization,
    Range,
    Referer,
    TE,
    UserAgent,

    // response
    AcceptRanges,
    Age,
    ETag,
    Location,
    RetryAfter,
    Server,
    Vary,

    AccessControlAllowOrigin,
    AccessControlAllowMethods,
    AccessControlAllowHeaders,
    AccessControlAllowCredentials,
    AccessControlMaxAge,

    // general
    CacheControl,
    Connection,
    Date,
    Trailer,
    TransferEncoding,
    Via,
    Warning,
} impl Header {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "Accept",
            Self::AcceptEncoding => "Accept-Encoding",
            Self::AcceptLanguage => "Accept-Language",
            Self::Authorization => "Authorization",
            Self::ContentType => "Content-Type",
            Self::Expect => "Expect",
            Self::From => "From",
            Self::Host => "Host",
            Self::IfMatch => "If-Match",
            Self::IfModifiedSince => "If-ModifiedSince",
            Self::IfNoneMatch => "If-NoneMatch",
            Self::IfRange => "If-Range",
            Self::IfUnmodifiedSince => "If-Unmodified-Since",
            Self::MaxForwords => "Max-Forwords",
            Self::ProxyAuthorization => "Proxy-Authorization",
            Self::Range => "Range",
            Self::Referer => "Referer",
            Self::TE => "TE",
            Self::UserAgent => "User-Agent",

            Self::AcceptRanges => "Accept-Ranges",
            Self::Age => "Age",
            Self::ETag => "E-Tag",
            Self::Location => "Location",
            Self::RetryAfter => "Retry-After",
            Self::Server => "Server",
            Self::Vary => "Vary",

            Self::AccessControlAllowOrigin => "Access-Control-Allow-Origin",
            Self::AccessControlAllowMethods => "Access-Control-Allow-Methods",
            Self::AccessControlAllowHeaders => "Access-Control-Allow-Headers",
            Self::AccessControlAllowCredentials => "Access-Control-Allow-Credentials",
            Self::AccessControlMaxAge => "Access-Control-Max-Age",

            Self::CacheControl => "Cache-Control",
            Self::Connection => "Connection",
            Self::Date => "Date",
            Self::Trailer => "Trailer",
            Self::TransferEncoding => "Transfer-Encoding",
            Self::Via => "Via",
            Self::Warning => "Warning",
        }
    }
}
