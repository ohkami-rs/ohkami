#[derive(PartialEq, Debug)]
pub(super) enum Pattern<'p> {
    Any,
    Str(&'p str),
    Param(&'p str /* param name */),
} impl<'p> Pattern<'p> {
    pub fn from(section: &'p str) -> Self {
        match section {
            "*" => Self::Any,
            p if p.starts_with(':') => Self::Param(&p[1..]),
            p => Self::Str(p),
        }
    }
    pub fn matches(&self, section: &'p str) -> (bool, Option<(&'p str, &'p str)>) {
        match self {
            Pattern::Any => (true, None),
            Pattern::Str(p) => (p == &section, None),
            Pattern::Param(name) => (true, Some((name, section))),
        }
    }
    pub fn is(&self, another: &Self) -> bool {
        match self {
            Self::Any => another.is_any(),
            Self::Str(p) => p == &another.as_str(),
            Self::Param(_) => another.is_param(),
        }
    }
    fn is_param(&self) -> bool {
        match self {
            Pattern::Param(_) => true,
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