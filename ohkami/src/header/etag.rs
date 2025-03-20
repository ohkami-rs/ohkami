use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub enum ETag<'header> {
    Any,
    Strong(Cow<'header, str>),
    Weak(Cow<'header, str>),
}

pub enum ETagError {
    InvalidFormat,
    InvalidCharactor,
}
impl std::fmt::Debug for ETagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ETagError::InvalidFormat => "InvalidFormat",
            ETagError::InvalidCharactor => "InvalidCharactor",
        })
    }
}
impl std::fmt::Display for ETagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ETagError::InvalidFormat => "InvalidFormat(Etag must be * or a strong/weak tag)",
            ETagError::InvalidCharactor => "InvalidCharactor(Etag can only contain ASCII characters)",
        })
    }
}
impl std::error::Error for ETagError {}

impl<'header> ETag<'header> {
    pub fn serialize(&self) -> Cow<'static, str> {
        match self {
            ETag::Any => Cow::Borrowed("*"),
            ETag::Strong(value) => Cow::Owned(format!("\"{value}\"")),
            ETag::Weak(value) => Cow::Owned(format!("W/\"{value}\"")),
        }
    }

    /// Parse a single ETag.
    pub fn parse(mut raw: &'header str) -> Result<Self, ETagError> {
        if raw == "*" {
            Ok(ETag::Any)
        } else {
            let is_weak = raw.starts_with("W/");
            if is_weak {
                raw = &raw[2..];
            }

            raw = (raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"'))
                .then(|| &raw[1..raw.len() - 1])
                .ok_or(ETagError::InvalidFormat)?;

            let _ = raw.is_ascii()
                .then_some(())
                .ok_or(ETagError::InvalidCharactor)?;

            Ok(if is_weak {
                ETag::Weak(Cow::Borrowed(raw))
            } else {
                ETag::Strong(Cow::Borrowed(raw))
            })
        }
    }

    /// Parse comma-separated ETags into an iterator of `Result<ETag, ETagError>`.
    /// Invalid ETag is returned as `Err`.
    pub fn try_iter_from(raw: &'header str) -> impl Iterator<Item = Result<Self, ETagError>> + 'header {
        raw.split(", ").map(ETag::parse)
    }

    /// Parse comma-separated ETags into an iterator of `ETag`.
    /// Invalid ETag is just ignored.
    /// 
    /// ## Example
    /// 
    /// ```
    /// use ohkami::header::ETag;
    /// 
    /// # fn main() {
    /// let mut etags = ETag::iter_from(
    ///     r#""abc123", W/"def456", "ghi789""#
    /// );
    /// 
    /// assert_eq!(etags.next(), Some(ETag::Strong("abc123".into())));
    /// assert_eq!(etags.next(), Some(ETag::Weak("def456".into())));
    /// assert_eq!(etags.next(), Some(ETag::Strong("ghi789".into())));
    /// assert_eq!(etags.next(), None);
    /// 
    /// let mut etags = ETag::iter_from("*");
    /// assert_eq!(etags.next(), Some(ETag::Any));
    /// assert_eq!(etags.next(), None);
    /// # }
    /// ```
    pub fn iter_from(raw: &'header str) -> impl Iterator<Item = Self> + 'header {
        raw.split(", ").filter_map(|it| ETag::parse(it).ok())
    }

    pub fn matches(&self, other: &ETag<'_>) -> bool {
        match (self, other) {
            (ETag::Any, _) | (_, ETag::Any) => true,
            | (ETag::Strong(a), ETag::Strong(b))
            | (ETag::Strong(a), ETag::Weak(b))
            | (ETag::Weak(a),   ETag::Strong(b))
            | (ETag::Weak(a),   ETag::Weak(b))
            => a == b,
        }
    }

    pub fn into_owned(self) -> ETag<'static> {
        match self {
            ETag::Any => ETag::Any,
            ETag::Strong(cow) => ETag::Strong(Cow::Owned(cow.into_owned())),
            ETag::Weak(cow) => ETag::Weak(Cow::Owned(cow.into_owned())),
        }
    }
}

impl ETag<'static> {
    pub fn new(value: String) -> Result<Self, ETagError> {
        value.is_ascii()
            .then_some(Self::Strong(value.into()))
            .ok_or(ETagError::InvalidCharactor)
    }
}