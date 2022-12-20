#[derive(PartialEq, Debug)]
pub(super) enum Pattern<'p> {
    Any,
    Param,
    Str(&'p str),
} impl<'p> Pattern<'p> {
    pub fn from(section: &'p str) -> Self {
        match section {
            "*" => Self::Any,
            p if p.starts_with(':') => Self::Param,
            p => Self::Str(p),
        }
    }
    pub fn matches(&self, section: &'p str) -> bool {
        match self {
            Pattern::Any => true,
            Pattern::Str(p) => p == &section,
            Pattern::Param => true,
        }
    }
    pub fn is(&self, another: &Self) -> bool {
        match self {
            Self::Any => another.is_any(),
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
    fn is_any(&self) -> bool {
        match self {
            Pattern::Any => true,
            _ => false,
        }
    }
}