use super::QValue;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Gzip,
    Deflate,
    Brotli,
    Zstd,
    Identity,
}

impl Encoding {
    pub const fn name(&self) -> &'static str {
        match self {
            Encoding::Gzip => "gzip",
            Encoding::Deflate => "deflate",
            Encoding::Brotli => "br",
            Encoding::Zstd => "zstd",
            Encoding::Identity => "identity",
        }
    }

    pub const fn parse(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"gzip" => Some(Encoding::Gzip),
            b"deflate" => Some(Encoding::Deflate),
            b"br" => Some(Encoding::Brotli),
            b"zstd" => Some(Encoding::Zstd),
            b"identity" => Some(Encoding::Identity),
            _ => None,
        }
    }

    pub const fn extension(&self) -> Option<&'static str> {
        match self {
            Encoding::Gzip => Some("gz"),
            Encoding::Deflate => Some("deflate"),
            Encoding::Brotli => Some("br"),
            Encoding::Zstd => Some("zst"),
            Encoding::Identity => None,
        }
    }

    pub fn from_extension(ext: &OsStr) -> Option<Self> {
        match ext.to_str()?.as_bytes() {
            b"gz" => Some(Encoding::Gzip),
            b"deflate" => Some(Encoding::Deflate),
            b"br" => Some(Encoding::Brotli),
            b"zst" => Some(Encoding::Zstd),
            _ => None,
        }
    }
}

pub enum CompressionEncoding {
    Single(Encoding),
    Multiple(Box<Vec<Encoding>>),
}

impl std::fmt::Debug for CompressionEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CompressionEncoding({})", self.to_content_encoding())
    }
}

impl CompressionEncoding {
    /// Create a `CompressionEncoding` from a file path and return it with
    /// the original path with compression-extensions removed.
    /// 
    /// The file path must be a file, and the encoding is determined by the file extension.
    /// For example, if the file path is `foo.txt.gz`, the encoding will be `Gzip`.
    /// If the file path has no encoding, `None` is returned.
    /// If the file path has multiple encodings, they will be returned in the order they were applied.
    /// For example, if the file path is `foo.txt.gz.br`, the encodings will be `Gzip` and `Brotli`.
    pub fn from_file_path(p: &Path) -> Option<(Self, PathBuf)> {
        if !p.is_file() {
            return None;
        }

        let mut p = p.to_owned();

        let mut encodings = Vec::new();
        while let Some(e) = p.extension().and_then(Encoding::from_extension) {
            encodings.push(e);
            p.set_extension("");
        }
        // Reverse to the order the encodings were applied
        encodings.reverse();

        match encodings.len() {
            0 => None,
            1 => Some((Self::Single(encodings.pop().unwrap()), p)),
            _ => Some((Self::Multiple(Box::new(encodings)), p)),
        }
    }

    pub fn to_extension(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Self::Single(encoding) => {
                encoding.extension().unwrap_or_default().into()
            }
            Self::Multiple(encodings) => {
                let ext = encodings.iter().flat_map(Encoding::extension).collect::<Vec<_>>().join(".");
                ext.into()
            }
        }
    }

    pub fn to_content_encoding(&self) -> std::borrow::Cow<'static, str> {
        match self {
            Self::Single(encoding) => encoding.name().into(),
            Self::Multiple(encodings) => {
                let enc = encodings.iter().map(Encoding::name).collect::<Vec<_>>().join(", ");
                enc.into()
            }
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptEncoding {
    gzip: QValue,
    deflate: QValue,
    br: QValue,
    zstd: QValue,
    identity: QValue,
}

impl AcceptEncoding {
    pub fn parse(s: &str) -> Self {
        enum EncodingOrAny {
            Encoding(Encoding),
            Any,
        }
        impl EncodingOrAny {
            #[inline]
            fn parse(s: &str) -> Option<Self> {
                match s {
                    "*" => Some(Self::Any),
                    _ => Encoding::parse(s).map(Self::Encoding),
                }
            }
        }

        let mut gzip = None;
        let mut deflate = None;
        let mut br = None;
        let mut zstd = None;
        let mut identity = None;

        let mut any = None;

        for part in s.split(',').map(str::trim) {
            let (encoding, q) = part
                .split_once(';')
                .and_then(|(encoding, q)| Some((EncodingOrAny::parse(encoding), QValue::parse(q)?)))
                .unwrap_or((EncodingOrAny::parse(part), QValue::default()));
            match encoding {
                Some(EncodingOrAny::Encoding(Encoding::Gzip)) => gzip = Some(q),
                Some(EncodingOrAny::Encoding(Encoding::Deflate)) => deflate = Some(q),
                Some(EncodingOrAny::Encoding(Encoding::Brotli)) => br = Some(q),
                Some(EncodingOrAny::Encoding(Encoding::Zstd)) => zstd = Some(q),
                Some(EncodingOrAny::Encoding(Encoding::Identity)) => identity = Some(q),
                Some(EncodingOrAny::Any) => any = Some(q),
                None => () // ignore unknown encodings
            }
        }

        if let Some(q) = any {
            if gzip.is_none() {gzip = Some(q);}
            if deflate.is_none() {deflate = Some(q);}
            if br.is_none() {br = Some(q);}
            if zstd.is_none() {zstd = Some(q);}
            if identity.is_none() {identity = Some(q);}
        }

        Self {
            gzip: gzip.unwrap_or(QValue(0)),
            deflate: deflate.unwrap_or(QValue(0)),
            br: br.unwrap_or(QValue(0)),
            zstd: zstd.unwrap_or(QValue(0)),
            // identity is acceptable unless explicitly set to q=0 in request,
            // but of course compressions are prefered if available.
            // so we set it to 1, minimum non-zero value.
            identity: identity.unwrap_or(QValue(1)),
        }
    }

    pub fn iter_in_preferred_order(&self) -> impl Iterator<Item = Encoding> {
        let mut encodings = [
            (self.gzip, Encoding::Gzip),
            (self.deflate, Encoding::Deflate),
            (self.br, Encoding::Brotli),
            (self.zstd, Encoding::Zstd),
            (self.identity, Encoding::Identity),
        ];
        encodings.sort_unstable_by(|(q1, _), (q2, _)| q2.cmp(q1));
        encodings.into_iter().filter_map(|(q, encoding)| (!q.is_zero()).then_some(encoding))
    }

    pub const fn accepts(&self, encoding: Encoding) -> bool {
        match encoding {
            Encoding::Gzip => !self.gzip.is_zero(),
            Encoding::Deflate => !self.deflate.is_zero(),
            Encoding::Brotli => !self.br.is_zero(),
            Encoding::Zstd => !self.zstd.is_zero(),
            Encoding::Identity => !self.identity.is_zero(),
        }
    }
    pub fn accepts_compression(&self, encoding: &CompressionEncoding) -> bool {
        match encoding {
            CompressionEncoding::Single(encoding) => self.accepts(*encoding),
            CompressionEncoding::Multiple(encodings) => encodings.iter().all(|e| self.accepts(*e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let accept_encoding = AcceptEncoding::parse("br");
        assert_eq!(accept_encoding.gzip, QValue(0));
        assert_eq!(accept_encoding.deflate, QValue(0));
        assert_eq!(accept_encoding.br, QValue(1000));
        assert_eq!(accept_encoding.zstd, QValue(0));
        assert_eq!(accept_encoding.identity, QValue(1));

        let accept_encoding = AcceptEncoding::parse("gzip, deflate, br");
        assert_eq!(accept_encoding.gzip, QValue(1000));
        assert_eq!(accept_encoding.deflate, QValue(1000));
        assert_eq!(accept_encoding.br, QValue(1000));
        assert_eq!(accept_encoding.zstd, QValue(0));
        assert_eq!(accept_encoding.identity, QValue(1));

        let accept_encoding = AcceptEncoding::parse("gzip;q=0.5, deflate;q=0.8, br;q=1.0");
        assert_eq!(accept_encoding.gzip, QValue(500));
        assert_eq!(accept_encoding.deflate, QValue(800));
        assert_eq!(accept_encoding.br, QValue(1000));
        assert_eq!(accept_encoding.zstd, QValue(0));
        assert_eq!(accept_encoding.identity, QValue(1));

        let accept_encoding = AcceptEncoding::parse("*");
        assert_eq!(accept_encoding.gzip, QValue(1000));
        assert_eq!(accept_encoding.deflate, QValue(1000));
        assert_eq!(accept_encoding.br, QValue(1000));
        assert_eq!(accept_encoding.zstd, QValue(1000));
        assert_eq!(accept_encoding.identity, QValue(1000));

        let accept_encoding = AcceptEncoding::parse("gzip;q=0.5, deflate;q=0.8, br;q=1.0, *;q=0.9");
        assert_eq!(accept_encoding.gzip, QValue(500));
        assert_eq!(accept_encoding.deflate, QValue(800));
        assert_eq!(accept_encoding.br, QValue(1000));
        assert_eq!(accept_encoding.zstd, QValue(900));
        assert_eq!(accept_encoding.identity, QValue(900));

        let accept_encoding = AcceptEncoding::parse("gzip;q=0.5, identity;q=0");
        assert_eq!(accept_encoding.gzip, QValue(500));
        assert_eq!(accept_encoding.deflate, QValue(0));
        assert_eq!(accept_encoding.br, QValue(0));
        assert_eq!(accept_encoding.zstd, QValue(0));
        // by explicitly set to q=0
        assert_eq!(accept_encoding.identity, QValue(0));
    }

    #[test]
    fn test_iter_in_preferred_order() {
        let accept_encoding = AcceptEncoding::parse("br");
        let mut iter = accept_encoding.iter_in_preferred_order();
        assert_eq!(iter.next(), Some(Encoding::Brotli));
        assert_eq!(iter.next(), Some(Encoding::Identity));
        assert_eq!(iter.next(), None);

        // gzip and deflate have the same qvalue, so the order is not guaranteed
        let accept_encoding = AcceptEncoding::parse("gzip, deflate, br");
        let encodings = accept_encoding.iter_in_preferred_order().collect::<Vec<_>>();
        assert_eq!(encodings.len(), 4);
        assert!(encodings.contains(&Encoding::Gzip));
        assert!(encodings.contains(&Encoding::Deflate));
        assert!(encodings.contains(&Encoding::Brotli));
        assert!(encodings.contains(&Encoding::Identity));

        let accept_encoding = AcceptEncoding::parse("gzip;q=0.5, deflate;q=0.8, br;q=1.0");
        let mut iter = accept_encoding.iter_in_preferred_order();
        assert_eq!(iter.next(), Some(Encoding::Brotli));
        assert_eq!(iter.next(), Some(Encoding::Deflate));
        assert_eq!(iter.next(), Some(Encoding::Gzip));
        assert_eq!(iter.next(), Some(Encoding::Identity));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_accept() {
        let accept_encoding = AcceptEncoding::default();
        assert!(accept_encoding.accepts(Encoding::Identity));
        assert!(accept_encoding.accepts(Encoding::Gzip));
        assert!(accept_encoding.accepts(Encoding::Deflate));
        assert!(accept_encoding.accepts(Encoding::Brotli));
        assert!(accept_encoding.accepts(Encoding::Zstd));

        let accept_encoding = AcceptEncoding::parse("br");
        // identity is accaptable unless explicitly set to q=0 in request
        assert!(accept_encoding.accepts(Encoding::Identity));
        assert!(accept_encoding.accepts(Encoding::Brotli));
        assert!(!accept_encoding.accepts(Encoding::Gzip));
        assert!(!accept_encoding.accepts(Encoding::Deflate));
        assert!(!accept_encoding.accepts(Encoding::Zstd));

        let accept_encoding = AcceptEncoding::parse("gzip, deflate, br");
        // identity is accaptable unless explicitly set to q=0 in request
        assert!(accept_encoding.accepts(Encoding::Identity));
        assert!(accept_encoding.accepts(Encoding::Gzip));
        assert!(accept_encoding.accepts(Encoding::Deflate));
        assert!(accept_encoding.accepts(Encoding::Brotli));
        assert!(!accept_encoding.accepts(Encoding::Zstd));

        let accept_encoding = AcceptEncoding::parse("gzip;q=0.5, deflate;q=0.8, br;q=1.0");
        // identity is accaptable unless explicitly set to q=0 in request
        assert!(accept_encoding.accepts(Encoding::Identity));
        assert!(accept_encoding.accepts(Encoding::Gzip));
        assert!(accept_encoding.accepts(Encoding::Deflate));
        assert!(accept_encoding.accepts(Encoding::Brotli));
        assert!(!accept_encoding.accepts(Encoding::Zstd));

        let accept_encoding = AcceptEncoding::parse("gzip;q=0.5, identity;q=0");
        // here identity is NOT acceptable because of explicit q=0
        assert!(!accept_encoding.accepts(Encoding::Identity));
        assert!(accept_encoding.accepts(Encoding::Gzip));
        assert!(!accept_encoding.accepts(Encoding::Brotli));
        assert!(!accept_encoding.accepts(Encoding::Deflate));
        assert!(!accept_encoding.accepts(Encoding::Zstd));
    }
}
