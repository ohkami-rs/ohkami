use std::borrow::Cow;

pub struct ETag<'header>(ETagVariant<'header>);

#[derive(Clone, Debug, PartialEq)]
enum ETagVariant<'header> {
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
            ETagError::InvalidCharactor => {
                "InvalidCharactor(Etag can only contain ASCII characters)"
            }
        })
    }
}
impl std::error::Error for ETagError {}

impl<'a> ETag<'a> {
    pub const fn is_any(&self) -> bool {
        matches!(self.0, ETagVariant::Any)
    }
    pub const fn is_strong(&self) -> bool {
        matches!(self.0, ETagVariant::Strong(_))
    }
    pub const fn is_weak(&self) -> bool {
        matches!(self.0, ETagVariant::Weak(_))
    }

    pub const fn any() -> Self {
        Self(ETagVariant::Any)
    }
    pub fn strong(value: impl Into<Cow<'a, str>>) -> Result<Self, ETagError> {
        let value = value.into();
        value
            .is_ascii()
            .then_some(Self(ETagVariant::Strong(value)))
            .ok_or(ETagError::InvalidCharactor)
    }
    pub fn weak(value: impl Into<Cow<'a, str>>) -> Result<Self, ETagError> {
        let value = value.into();
        value
            .is_ascii()
            .then_some(Self(ETagVariant::Weak(value)))
            .ok_or(ETagError::InvalidCharactor)
    }
}

impl<'header> ETag<'header> {
    pub fn serialize(&self) -> Cow<'static, str> {
        match &self.0 {
            ETagVariant::Any => Cow::Borrowed("*"),
            ETagVariant::Strong(value) => Cow::Owned(format!("\"{value}\"")),
            ETagVariant::Weak(value) => Cow::Owned(format!("W/\"{value}\"")),
        }
    }

    /// Parse a single ETag.
    pub fn parse(mut raw: &'header str) -> Result<Self, ETagError> {
        if raw == "*" {
            Ok(Self(ETagVariant::Any))
        } else {
            let is_weak = raw.starts_with("W/");
            if is_weak {
                raw = &raw[2..];
            }

            raw = (raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"'))
                .then(|| &raw[1..raw.len() - 1])
                .ok_or(ETagError::InvalidFormat)?;

            if !raw.is_ascii() {
                return Err(ETagError::InvalidCharactor);
            }

            Ok(if is_weak {
                Self(ETagVariant::Weak(Cow::Borrowed(raw)))
            } else {
                Self(ETagVariant::Strong(Cow::Borrowed(raw)))
            })
        }
    }

    /// Parse comma-separated ETags into an iterator of `Result<ETag, ETagError>`.
    /// Invalid ETag is returned as `Err`.
    pub fn try_iter_from(
        raw: &'header str,
    ) -> impl Iterator<Item = Result<Self, ETagError>> + 'header {
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
        match (&self.0, &other.0) {
            (ETagVariant::Any, _) | (_, ETagVariant::Any) => true,
            (ETagVariant::Strong(a), ETagVariant::Strong(b))
            | (ETagVariant::Strong(a), ETagVariant::Weak(b))
            | (ETagVariant::Weak(a), ETagVariant::Strong(b))
            | (ETagVariant::Weak(a), ETagVariant::Weak(b)) => a == b,
        }
    }

    pub fn into_owned(self) -> ETag<'static> {
        match self.0 {
            ETagVariant::Any => ETag::any(),
            ETagVariant::Strong(cow) => ETag::strong(cow.into_owned()).unwrap(),
            ETagVariant::Weak(cow) => ETag::weak(cow.into_owned()).unwrap(),
        }
    }
}
