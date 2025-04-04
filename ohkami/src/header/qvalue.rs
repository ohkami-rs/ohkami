// The QValue struct is used to represent the quality value of an encoding.
// It is a wrapper around a u16 value, which represents the quality value
// as a real number with at most 3 decimal places.
// For example, a QValue of 0.5 would be represented as 500.
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub struct QValue(pub(crate) u16);

impl QValue {
    pub fn parse(s: &str) -> Option<Self> {
        let mut r = byte_reader::Reader::new(s.as_bytes());
        match r.consume_oneof(["q=0", "q=1"])? {
            0 => {
                let mut q = 0;
                if r.consume(".").is_some() {
                    for factor in [100, 10, 1] {
                        if let Some(b) = r.next_if(u8::is_ascii_digit) {
                            q += factor * (b - b'0') as u16;
                        } else {
                            break;
                        }
                    }
                }
                Some(Self(q))
            },
            1 => Some(Self(1000)),
            _ => unreachable!(),
        }
    }

    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Default for QValue {
    fn default() -> Self {
        Self(1000)
    }
}

impl std::fmt::Debug for QValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "q={}", self.0 as f32 / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qvalue_parse() {
        assert_eq!(QValue::parse("q=0.5"), Some(QValue(500)));
        assert_eq!(QValue::parse("q=1"), Some(QValue(1000)));
        assert_eq!(QValue::parse("q=0"), Some(QValue(0)));
        assert_eq!(QValue::parse("q=0.123"), Some(QValue(123)));
        assert_eq!(QValue::parse("q=0.999"), Some(QValue(999)));
        assert_eq!(QValue::parse("q=0.000"), Some(QValue(0)));
        assert_eq!(QValue::parse("q=1.000"), Some(QValue(1000)));
    }
}
