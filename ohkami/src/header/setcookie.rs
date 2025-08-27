use std::borrow::Cow;


#[derive(Debug, PartialEq)]
pub enum SameSitePolicy {
    Strict,
    Lax,
    None,
}
impl SameSitePolicy {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Strict => "Strict",
            Self::Lax    => "Lax",
            Self::None   => "None",
        }
    }
    const fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"Strict" | b"strict" => Some(Self::Strict),
            b"Lax" | b"lax" => Some(Self::Lax),
            b"None" | b"none" => Some(Self::None),
            _ => None
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SetCookie<'c> {
    pub(crate) cookie:   (&'c str, Cow<'c, str>),
    pub(crate) expires:  Option<Cow<'c, str>>,
    pub(crate) max_age:   Option<u64>,
    pub(crate) domain:   Option<Cow<'c, str>>,
    pub(crate) path:     Option<Cow<'c, str>>,
    pub(crate) secure:   Option<bool>,
    pub(crate) http_only: Option<bool>,
    pub(crate) same_site: Option<SameSitePolicy>,
}
impl<'c> SetCookie<'c> {
    pub fn cookie(&self) -> (&str, &str) {
        let (name, value) = &self.cookie;
        (name, &value)
    }
    pub fn expires(&self) -> Option<&str> {
        self.expires.as_deref()
    }
    pub const fn max_age(&self) -> Option<u64> {
        self.max_age
    }
    pub fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
    pub const fn secure(&self) -> Option<bool> {
        self.secure
    }
    pub const fn http_only(&self) -> Option<bool> {
        self.http_only
    }
    /// `Some`: `"Lax" | "None" | "Strict"`
    pub const fn same_site(&self) -> Option<&'static str> {
        match &self.same_site {
            None         => None,
            Some(policy) => Some(policy.as_str())
        }
    }

    pub(crate) fn from_raw(str: &'c str) -> Result<Self, String> {
        let mut r = byte_reader::Reader::new(str.as_bytes());

        let mut this = {
            let name  = std::str::from_utf8(r.read_until(b"=")).map_err(|e| format!("Invalid Cookie name: {e}"))?;
            r.consume("=").ok_or_else(|| format!("No `=` found in a `Set-Cookie` header value"))?;
            let value =  ohkami_lib::percent_decode_utf8({
                let mut bytes = r.read_until(b"; ");
                let len = bytes.len();
                if len >= 2 && bytes[0] == b'"' && bytes[len-1] == b'"' {
                    bytes = &bytes[1..(len-1)]
                }
                bytes
            }).map_err(|e| format!("Invalid Cookie value: {e}"))?;

            Self {
                cookie: (name, value),
                expires: None,
                max_age: None,
                domain: None,
                path: None,
                same_site: None,
                secure: None,
                http_only: None,
            }
        };

        while r.consume("; ").is_some() {
            let directive = r.read_until(b"; ");
            let mut r = byte_reader::Reader::new(directive);
            match r.consume_oneof([
                "Expires", "Max-Age", "Domain", "Path", "SameSite", "Secure", "HttpOnly"
            ]) {
                Some(0) => {
                    r.consume("=").ok_or_else(|| format!("Invalid `Expires`: No `=` found"))?;
                    let value = std::str::from_utf8(r.read_until(b"; ")).map_err(|e| format!("Invalid `Expires`: {e}"))?;
                    this.expires = Some(Cow::Borrowed(value))
                },
                Some(1) => {
                    r.consume("=").ok_or_else(|| format!("Invalid `Max-Age`: No `=` found"))?;
                    let value = r.read_until(b"; ").iter().fold(0, |secs, d| 10*secs + (*d - b'0') as u64);
                    this.max_age = Some(value)
                }
                Some(2) => {
                    r.consume("=").ok_or_else(|| format!("Invalid `Domain`: No `=` found"))?;
                    let value = std::str::from_utf8(r.read_until(b"; ")).map_err(|e| format!("Invalid `Domain`: {e}"))?;
                    this.domain = Some(Cow::Borrowed(value))
                },
                Some(3) => {
                    r.consume("=").ok_or_else(|| format!("Invalid `Path`: No `=` found"))?;
                    let value = std::str::from_utf8(r.read_until(b"; ")).map_err(|e| format!("Invalid `Path`: {e}"))?;
                    this.path = Some(Cow::Borrowed(value))
                }
                Some(4) => {
                    r.consume("=").ok_or_else(|| format!("Invalid `SameSite`: No `=` found"))?;
                    this.same_site = SameSitePolicy::from_bytes(r.read_until(b"; "));
                }
                Some(5) => this.secure = Some(true),
                Some(6) => this.http_only = Some(true),
                _ => return Err((|| format!("Unkown directive: `{}`", r.remaining().escape_ascii()))())
            }
        }

        Ok(this)
    }
}

pub struct SetCookieBuilder(SetCookie<'static>);
impl SetCookieBuilder {
    #[inline]
    pub(crate) fn new(cookie_name: &'static str, cookie_value: impl Into<Cow<'static, str>>) -> Self {
        Self(SetCookie {
            cookie: (cookie_name, cookie_value.into()),
            expires: None, max_age: None, domain: None, path: None, secure: None, http_only: None, same_site: None,
        })
    }
    pub(crate) fn build(self) -> String {
        let mut bytes = Vec::new();

        let (name, value) = self.0.cookie; {
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(b'=');
            bytes.extend_from_slice(ohkami_lib::percent_encode(&value).as_bytes());
        }
        if let Some(Expires) = self.0.expires {
            bytes.extend_from_slice(b"; Expires=");
            bytes.extend_from_slice(Expires.as_bytes());
        }
        if let Some(MaxAge) = self.0.max_age {
            bytes.extend_from_slice(b"; Max-Age=");
            bytes.extend_from_slice(MaxAge.to_string().as_bytes());
        }
        if let Some(Domain) = self.0.domain {
            bytes.extend_from_slice(b"; Domain=");
            bytes.extend_from_slice(Domain.as_bytes());
        }
        if let Some(Path) = self.0.path {
            bytes.extend_from_slice(b"; Path=");
            bytes.extend_from_slice(Path.as_bytes());
        }
        if let Some(true) = self.0.secure {
            bytes.extend_from_slice(b"; Secure");
        }
        if let Some(true) = self.0.http_only {
            bytes.extend_from_slice(b"; HttpOnly");
        }
        if let Some(SameSite) = self.0.same_site {
            bytes.extend_from_slice(b"; SameSite=");
            bytes.extend_from_slice(SameSite.as_str().as_bytes());
        }

        unsafe {// SAFETY: All fields and punctuaters is UTF-8
            String::from_utf8_unchecked(bytes)
        }
    }

    #[inline]
    pub fn expires(mut self, expires: impl Into<Cow<'static, str>>) -> Self {
        self.0.expires = Some(expires.into());
        self
    }
    #[inline]
    pub const fn max_age(mut self, max_age: u64) -> Self {
        self.0.max_age = Some(max_age);
        self
    }
    #[inline]
    pub fn domain(mut self, domain: impl Into<Cow<'static, str>>) -> Self {
        self.0.domain = Some(domain.into());
        self
    }
    #[inline]
    pub fn path(mut self, path: impl Into<Cow<'static, str>>) -> Self {
        self.0.path = Some(path.into());
        self
    }
    #[inline]
    pub const fn secure(mut self, yes: bool) -> Self {
        self.0.secure = Some(yes);
        self
    }
    #[inline]
    pub const fn http_only(mut self) -> Self {
        self.0.http_only = Some(true);
        self
    }
    #[inline]
    pub const fn same_site_lax(mut self) -> Self {
        self.0.same_site = Some(SameSitePolicy::Lax);
        self
    }
    #[inline]
    pub const fn same_site_none(mut self) -> Self {
        self.0.same_site = Some(SameSitePolicy::None);
        self
    }
    #[inline]
    pub const fn same_site_strict(mut self) -> Self {
        self.0.same_site = Some(SameSitePolicy::Strict);
        self
    }
}
