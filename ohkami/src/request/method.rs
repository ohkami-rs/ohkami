#[derive(Clone, Copy, PartialEq)]
pub enum Method {
    GET,
    PUT,
    POST,
    PATCH,
    DELETE,
    QUERY,
    HEAD,
    OPTIONS,
}

impl Method {
    pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"GET" => Some(Self::GET),
            b"PUT" => Some(Self::PUT),
            b"POST" => Some(Self::POST),
            b"PATCH" => Some(Self::PATCH),
            b"DELETE" => Some(Self::DELETE),
            b"QUERY" => Some(Self::QUERY),
            b"HEAD" => Some(Self::HEAD),
            b"OPTIONS" => Some(Self::OPTIONS),
            _ => None,
        }
    }
    // NOTE: `worker` v0.9.0 does not support `QUERY`.
    #[cfg(feature = "rt_worker")]
    #[inline(always)]
    pub(crate) const fn from_worker(w: ::worker::Method) -> Option<Self> {
        match w {
            ::worker::Method::Get => Some(Self::GET),
            ::worker::Method::Put => Some(Self::PUT),
            ::worker::Method::Post => Some(Self::POST),
            ::worker::Method::Patch => Some(Self::PATCH),
            ::worker::Method::Delete => Some(Self::DELETE),
            ::worker::Method::Head => Some(Self::HEAD),
            ::worker::Method::Options => Some(Self::OPTIONS),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::GET => "GET",
            Self::PUT => "PUT",
            Self::POST => "POST",
            Self::PATCH => "PATCH",
            Self::DELETE => "DELETE",
            Self::QUERY => "QUERY",
            Self::HEAD => "HEAD",
            Self::OPTIONS => "OPTIONS",
        }
    }

    #[inline]
    pub const fn is_safe(&self) -> bool {
        matches!(self, Self::GET | Self::HEAD | Self::OPTIONS)
    }
}

#[allow(non_snake_case)]
impl Method {
    pub const fn isGET(&self) -> bool {
        matches!(self, Method::GET)
    }
    pub const fn isPUT(&self) -> bool {
        matches!(self, Method::PUT)
    }
    pub const fn isPOST(&self) -> bool {
        matches!(self, Method::POST)
    }
    pub const fn isPATCH(&self) -> bool {
        matches!(self, Method::PATCH)
    }
    pub const fn isDELETE(&self) -> bool {
        matches!(self, Method::DELETE)
    }
    pub const fn isHEAD(&self) -> bool {
        matches!(self, Method::HEAD)
    }
    pub const fn isOPTIONS(&self) -> bool {
        matches!(self, Method::OPTIONS)
    }
}

const _: () = {
    impl std::fmt::Display for Method {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }

    impl std::fmt::Debug for Method {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
        }
    }

    impl<'de> serde::Deserialize<'de> for Method {
        #[inline]
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let s = <&'de str>::deserialize(d)?;
            Method::from_bytes(s.as_bytes())
                .ok_or_else(|| serde::de::Error::custom("unknown HTTP method"))
        }
    }
};
