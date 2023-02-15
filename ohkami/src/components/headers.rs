use std::{collections::BTreeMap, ops::Index, borrow::Cow};
use crate::{prelude::Result, utils::string::string};


pub struct RequestHeaders<'buf>(
    BTreeMap<&'buf str, Cow<'buf, str>>
);
pub struct ResponseHeaders(
    String // BTreeMap<&'static str, string>
);

const _: (/* RequestHeaders impls */) = {
    // impl From

    impl<'buf> Index for RequestHeaders<'buf> {
        type Output = Result<&'buf str>;
        fn index(&self, key: &str) -> &Self::Output {
            self.0.get(key)
        }
    }
};

const _: (/* ResponseHeaders impls */) = {
    impl ResponseHeaders {

    }
};

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
