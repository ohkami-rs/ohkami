#[derive(Clone, Copy, PartialEq)]
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
    #[inline] pub(crate) fn from_bytes(bytes: &[u8]) -> Self {
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
#[allow(non_snake_case)] impl Method {
    #[inline(always)] pub fn isGET(&self) -> bool {
        matches!(self, Method::GET)
    }
    #[inline(always)] pub fn isPUT(&self) -> bool {
        matches!(self, Method::PUT)
    }
    #[inline(always)] pub fn isPOST(&self) -> bool {
        matches!(self, Method::POST)
    }
    #[inline(always)] pub fn isPATCH(&self) -> bool {
        matches!(self, Method::PATCH)
    }
    #[inline(always)] pub fn isDELETE(&self) -> bool {
        matches!(self, Method::DELETE)
    }
    #[inline(always)] pub fn isHEAD(&self) -> bool {
        matches!(self, Method::HEAD)
    }
    #[inline(always)] pub fn isOPTIONS(&self) -> bool {
        matches!(self, Method::OPTIONS)
    }
}

const _: () = {
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
    
    impl std::fmt::Debug for Method {
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
};
