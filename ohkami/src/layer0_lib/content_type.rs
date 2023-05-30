use std::hint::unreachable_unchecked;

#[cfg_attr(test, derive(Clone, Debug, PartialEq))]
pub enum ContentType {
    JSON,
    URLEncoded,
    Text,
    HTML,
    Form {boundary: String},
}

impl ContentType {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"application/json"                  => Some(Self::JSON),
            b"application/x-www-form-urlencoded" => Some(Self::URLEncoded),
            b"text/plain"                        => Some(Self::Text),
            _ => bytes.strip_prefix(b"multipart/form-data; boundary=")
                .map(|bound| Self::Form {
                    boundary: unsafe {String::from_utf8_unchecked(bound.to_vec())}
                })
        }
    }

    pub(crate) fn into_bytes(&self) -> &[u8] {
        match self {
            Self::JSON => b"application/json",
            Self::Text => b"text/plain",
            Self::HTML => b"text/html",
            _ => unsafe {unreachable_unchecked()}
        }
    }
}
