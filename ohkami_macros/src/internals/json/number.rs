use std::fmt::Debug;

pub(crate) enum Number {
    Positive(usize),
    Negative(isize),
    Float(f64),
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
