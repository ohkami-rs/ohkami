use super::QValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Gzip,
    Deflate,
    Brotli,
    Zstd,
    Identity,
}

impl Encoding {
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

    pub const fn name(&self) -> &'static str {
        match self {
            Encoding::Gzip => "gzip",
            Encoding::Deflate => "deflate",
            Encoding::Brotli => "br",
            Encoding::Zstd => "zstd",
            Encoding::Identity => "identity",
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
}

pub struct AcceptEncoding {
    gzip: QValue,
    deflate: QValue,
    br: QValue,
    zstd: QValue,
    identity: QValue,
}

impl AcceptEncoding {
    pub fn parse(value: &str) -> Self {
        enum EncodingOrAny {
            Encoding(Encoding),
            Any,
        }
        impl EncodingOrAny {
            fn parse(value: &str) -> Option<Self> {
                match value {
                    "*" => Some(Self::Any),
                    _ => Encoding::parse(value).map(Self::Encoding),
                }
            }
        }

        let mut gzip = None;
        let mut deflate = None;
        let mut br = None;
        let mut zstd = None;
        let mut identity = None;

        let mut any = None;

        for part in value.split(',').map(str::trim) {
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
            identity: identity.unwrap_or(QValue(0)),
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
        assert_eq!(accept_encoding.identity, QValue(0));

        let accept_encoding = AcceptEncoding::parse("gzip, deflate, br");
        assert_eq!(accept_encoding.gzip, QValue(1000));
        assert_eq!(accept_encoding.deflate, QValue(1000));
        assert_eq!(accept_encoding.br, QValue(1000));
        assert_eq!(accept_encoding.zstd, QValue(0));
        assert_eq!(accept_encoding.identity, QValue(0));

        let accept_encoding = AcceptEncoding::parse("gzip;q=0.5, deflate;q=0.8, br;q=1.0");
        assert_eq!(accept_encoding.gzip, QValue(500));
        assert_eq!(accept_encoding.deflate, QValue(800));
        assert_eq!(accept_encoding.br, QValue(1000));
        assert_eq!(accept_encoding.zstd, QValue(0));
        assert_eq!(accept_encoding.identity, QValue(0));

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
    }

    #[test]
    fn test_iter_in_preferred_order() {
        let accept_encoding = AcceptEncoding::parse("br");
        let mut iter = accept_encoding.iter_in_preferred_order();
        assert_eq!(iter.next(), Some(Encoding::Brotli));
        assert_eq!(iter.next(), None);

        // gzip and deflate have the same qvalue, so the order is not guaranteed
        let accept_encoding = AcceptEncoding::parse("gzip, deflate, br");
        let encodings = accept_encoding.iter_in_preferred_order().collect::<Vec<_>>();
        assert_eq!(encodings.len(), 3);
        assert!(encodings.contains(&Encoding::Gzip));
        assert!(encodings.contains(&Encoding::Deflate));
        assert!(encodings.contains(&Encoding::Brotli));

        let accept_encoding = AcceptEncoding::parse("gzip;q=0.5, deflate;q=0.8, br;q=1.0");
        let mut iter = accept_encoding.iter_in_preferred_order();
        assert_eq!(iter.next(), Some(Encoding::Brotli));
        assert_eq!(iter.next(), Some(Encoding::Deflate));
        assert_eq!(iter.next(), Some(Encoding::Gzip));
        assert_eq!(iter.next(), None);
    }
}
