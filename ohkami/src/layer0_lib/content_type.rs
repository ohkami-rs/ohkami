use std::hint::unreachable_unchecked;

#[cfg_attr(test, derive(Clone, PartialEq))]
#[derive(Debug)]
pub enum ContentType {
    JSON,
    URLEncoded,
    Text,
    HTML,
    FormData { boundary: String },
}

impl ContentType {
    #[inline] pub fn is_json(&self) -> bool {
        match self {
            Self::JSON => true,
            _          => false,
        }
    }

    pub fn is_form(&self) -> bool {
        match self {
            Self::FormData { .. } => true,
            _                     => false,
        }
    }

    pub fn is_urlencoded(&self) -> bool {
        match self {
            Self::URLEncoded => true,
            _ =>                false,
        }
    }
}

impl ContentType {
    #[inline(always)] pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"application/json"                  => Some(Self::JSON),
            b"application/x-www-form-urlencoded" => Some(Self::URLEncoded),
            b"text/plain"                        => Some(Self::Text),
            _ => bytes.strip_prefix(b"multipart/form-data; boundary=")
                .map(|bound| Self::FormData {
                    boundary: unsafe {String::from_utf8_unchecked(bound.to_vec())}
                })
        }
    }

    #[inline(always)] pub(crate) const fn as_str(&self) -> &'static str {
        match self {
            Self::JSON => "application/json; charset=utf-8",
            Self::Text => "text/plain; charset=utf-8",
            Self::HTML => "text/html; charset=utf-8",
            _ => unsafe {unreachable_unchecked()}
        }
    }

    #[inline(always)] pub(crate) const fn as_bytes(&self) -> &'static [u8] {
        self.as_str().as_bytes()
    }
}
