#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Method {
    GET,
    PUT,
    POST,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
}

impl Method {
    #[inline(always)] pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        match bytes {
            b"GET"     => Self::GET,
            b"PUT"     => Self::PUT,
            b"POST"    => Self::POST,
            b"PATCH"   => Self::PATCH,
            b"DELETE"  => Self::DELETE,
            b"HEAD"    => Self::HEAD,
            b"OPTIONS" => Self::OPTIONS,
            _ => unreachable!("unknown method: `{}`", unsafe {std::str::from_utf8_unchecked(bytes)})
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::GET     => "GET",
            Self::PUT     => "PUT",
            Self::POST    => "POST",
            Self::PATCH   => "PATCH",
            Self::DELETE  => "DELETE",
            Self::HEAD    => "HEAD",
            Self::OPTIONS => "OPTIONS",
        })
    }
}
