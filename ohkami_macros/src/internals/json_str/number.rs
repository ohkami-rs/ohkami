use std::fmt::Debug;

pub(super) enum Number {
    Positive(usize),
    Negative(isize),
    Float(f64),
} impl Number {
    pub(super) fn to_string(self) -> String {
        match self {
            Self::Positive(p) => p.to_string(),
            Self::Negative(n) => n.to_string(),
            Self::Float(f) => f.to_string(),
        }
    }
}

impl Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Positive(p) => write!(f, "{p}"),
            Self::Negative(n) => write!(f, "{n}"),
            Self::Float(float) => write!(f, "{float}"),
        }
    }
}
