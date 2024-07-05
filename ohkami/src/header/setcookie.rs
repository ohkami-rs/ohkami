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
            b"Strict" => Some(Self::Strict),
            b"Lax"    => Some(Self::Lax),
            b"None"   => Some(Self::None),
            _ => None
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SetCookie<'c> {
    pub(crate) Cookie:   (&'c str, Cow<'c, str>),
    pub(crate) Expires:  Option<Cow<'c, str>>,
    pub(crate) MaxAge:   Option<u64>,
    pub(crate) Domain:   Option<Cow<'c, str>>,
    pub(crate) Path:     Option<Cow<'c, str>>,
    pub(crate) Secure:   Option<bool>,
    pub(crate) HttpOnly: Option<bool>,
    pub(crate) SameSite: Option<SameSitePolicy>,
}
impl<'c> SetCookie<'c> {
    pub fn Cookie(&self) -> (&str, &str) {
        let (name, value) = &self.Cookie;
        (name, &value)
    }
    pub fn Expires(&self) -> Option<&str> {
        self.Expires.as_deref()
    }
    pub const fn MaxAge(&self) -> Option<u64> {
        self.MaxAge
    }
    pub fn Domain(&self) -> Option<&str> {
        self.Domain.as_deref()
    }
    pub fn Path(&self) -> Option<&str> {
        self.Path.as_deref()
    }
    pub const fn Secure(&self) -> Option<bool> {
        self.Secure
    }
    pub const fn HttpOnly(&self) -> Option<bool> {
        self.HttpOnly
    }
    /// `Some`: `"Lax" | "None" | "Strict"`
    pub const fn SameSite(&self) -> Option<&'static str> {
        match &self.SameSite {
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
                Cookie: (name, value),
                Expires:  None,
                MaxAge:   None,
                Domain:   None,
                Path:     None,
                SameSite: None,
                Secure:   None,
                HttpOnly: None,
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
                    this.Expires = Some(Cow::Borrowed(value))
                },
                Some(1) => {
                    r.consume("=").ok_or_else(|| format!("Invalid `Max-Age`: No `=` found"))?;
                    let value = r.read_until(b"; ").iter().fold(0, |secs, d| 10*secs + (*d - b'0') as u64);
                    this.MaxAge = Some(value)
                }
                Some(2) => {
                    r.consume("=").ok_or_else(|| format!("Invalid `Domain`: No `=` found"))?;
                    let value = std::str::from_utf8(r.read_until(b"; ")).map_err(|e| format!("Invalid `Domain`: {e}"))?;
                    this.Domain = Some(Cow::Borrowed(value))
                },
                Some(3) => {
                    r.consume("=").ok_or_else(|| format!("Invalid `Path`: No `=` found"))?;
                    let value = std::str::from_utf8(r.read_until(b"; ")).map_err(|e| format!("Invalid `Path`: {e}"))?;
                    this.Path = Some(Cow::Borrowed(value))
                }
                Some(4) => {
                    r.consume("=").ok_or_else(|| format!("Invalid `SameSite`: No `=` found"))?;
                    this.SameSite = SameSitePolicy::from_bytes(r.read_until(b"; "));
                }
                Some(5) => this.Secure = Some(true),
                Some(6) => this.HttpOnly = Some(true),
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
            Cookie: (cookie_name, cookie_value.into()),
            Expires: None, MaxAge: None, Domain: None, Path: None, Secure: None, HttpOnly: None, SameSite: None,
        })
    }
    pub(crate) fn build(self) -> String {
        let mut bytes = Vec::new();

        let (name, value) = self.0.Cookie; {
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(b'=');
            bytes.extend_from_slice(ohkami_lib::percent_encode(&value).as_bytes());
        }
        if let Some(Expires) = self.0.Expires {
            bytes.extend_from_slice(b"; Expires=");
            bytes.extend_from_slice(Expires.as_bytes());
        }
        if let Some(MaxAge) = self.0.MaxAge {
            bytes.extend_from_slice(b"; Max-Age=");
            bytes.extend_from_slice(MaxAge.to_string().as_bytes());
        }
        if let Some(Domain) = self.0.Domain {
            bytes.extend_from_slice(b"; Domain=");
            bytes.extend_from_slice(Domain.as_bytes());
        }
        if let Some(Path) = self.0.Path {
            bytes.extend_from_slice(b"; Path=");
            bytes.extend_from_slice(Path.as_bytes());
        }
        if let Some(true) = self.0.Secure {
            bytes.extend_from_slice(b"; Secure");
        }
        if let Some(true) = self.0.HttpOnly {
            bytes.extend_from_slice(b"; HttpOnly");
        }
        if let Some(SameSite) = self.0.SameSite {
            bytes.extend_from_slice(b"; SameSite=");
            bytes.extend_from_slice(SameSite.as_str().as_bytes());
        }

        unsafe {// SAFETY: All fields and punctuaters is UTF-8
            String::from_utf8_unchecked(bytes)
        }
    }

    #[inline]
    pub fn Expires(mut self, Expires: impl Into<Cow<'static, str>>) -> Self {
        self.0.Expires = Some(Expires.into());
        self
    }
    #[inline]
    pub const fn MaxAge(mut self, MaxAge: u64) -> Self {
        self.0.MaxAge = Some(MaxAge);
        self
    }
    #[inline]
    pub fn Domain(mut self, Domain: impl Into<Cow<'static, str>>) -> Self {
        self.0.Domain = Some(Domain.into());
        self
    }
    #[inline]
    pub fn Path(mut self, Path: impl Into<Cow<'static, str>>) -> Self {
        self.0.Path = Some(Path.into());
        self
    }
    #[inline]
    pub const fn Secure(mut self) -> Self {
        self.0.Secure = Some(true);
        self
    }
    #[inline]
    pub const fn HttpOnly(mut self) -> Self {
        self.0.HttpOnly = Some(true);
        self
    }
    #[inline]
    pub const fn SameSiteLax(mut self) -> Self {
        self.0.SameSite = Some(SameSitePolicy::Lax);
        self
    }
    #[inline]
    pub const fn SameSiteNone(mut self) -> Self {
        self.0.SameSite = Some(SameSitePolicy::None);
        self
    }
    #[inline]
    pub const fn SameSiteStrict(mut self) -> Self {
        self.0.SameSite = Some(SameSitePolicy::Strict);
        self
    }
}
