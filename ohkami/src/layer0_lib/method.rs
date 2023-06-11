#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Method {
    GET,
    PUT,
    POST,
    HEAD,
    PATCH,
    DELETE,
    OPTIONS,
}

impl Method {
    #[inline(always)] pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
        match bytes {
            b"GET" => Self::GET,
            b"PUT" => Self::PUT,
            b"POST" => Self::POST,
            b"HEAD" => Self::HEAD,
            b"PATCH" => Self::PATCH,
            b"DELETE" => Self::DELETE,
            b"OPTIONS" => Self::OPTIONS,
            _ => unreachable!("unknown method: `{}`", unsafe {std::str::from_utf8_unchecked(bytes)})
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::GET => "GET",
            Self::PUT => "PUT",
            Self::POST => "POST",
            Self::HEAD => "HEAD",
            Self::PATCH => "PATCH",
            Self::DELETE => "DELETE",
            Self::OPTIONS => "OPTIONS",
        })
    }
}
