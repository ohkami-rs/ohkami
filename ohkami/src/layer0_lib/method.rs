#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Method {
    GET,
    PUT,
    POST,
    HEAD,
    PATCH,
    DELETE,
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
            _ => unreachable!("unknown method: `{}`", unsafe {std::str::from_utf8_unchecked(bytes)})
        }
    }
}
