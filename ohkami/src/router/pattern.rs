#[derive(PartialEq, Debug)]
pub(super) enum Pattern {
    Nil,
    Param,
    Str(&'static str),
} impl Pattern {
    pub fn from(section: &'static str) -> Self {
        match section {
            p if p.starts_with(':') => Self::Param,
            p => Self::Str(p),
        }
    }
    pub fn matches(&self, section: &str) -> bool {
        match self {
            Pattern::Nil => false,
            Pattern::Str(p) => p == &section,
            Pattern::Param => true,
        }
    }
    pub fn is(&self, another: &Self) -> bool {
        match self {
            Self::Nil => another.is_nil(),
            Self::Str(p) => p == &another.as_str(),
            Self::Param => another.is_param(),
        }
    }
    pub(crate) fn is_param(&self) -> bool {
        match self {
            Pattern::Param => true,
            _ => false,
        }
    }
    fn as_str(&self) -> &str {
        match self {
            Pattern::Str(p) => p,
            _ => unreachable!("`as_str` was called by Pattern other than `Str`"),
        }
    }
    pub(super) fn is_nil(&self) -> bool {
        match self {
            Pattern::Nil => true,
            _ => false,
        }
    }
}