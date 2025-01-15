//! serde case specifiers
//!
//! based on https://github.com/serde-rs/serde/blob/930401b0dd58a809fce34da091b8aa3d6083cb33/serde_derive/src/internals/case.rs

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Case {
    Lower,
    Upper,
    Pascal,
    Camel,
    Snake,
    ScreamingSnake,
    Kebab,
    ScreamingKebab,
}

impl From<String> for Case {
    fn from(s: String) -> Self {
        Self::from_str(&s).expect("unexpected case specifier")
    }
}

impl Case {
    pub(crate) const fn from_str(s: &str) -> Option<Self> {
        match s.as_bytes() {
            b"lowercase"            => Some(Self::Lower),
            b"UPPERCASE"            => Some(Self::Upper),
            b"PascalCase"           => Some(Self::Pascal),
            b"camelCase"            => Some(Self::Camel),
            b"snake_case"           => Some(Self::Snake),
            b"SCREAMING_SNAKE_CASE" => Some(Self::ScreamingSnake),
            b"kebab-case"           => Some(Self::Kebab),
            b"SCREAMING-KEBAB-CASE" => Some(Self::ScreamingKebab),
            _ => None
        }
    }

    pub(crate) fn apply_to_field(self, field: &str) -> String {
        match self {
            Self::Lower | Self::Snake => field.to_string(),
            Self::Upper => field.to_ascii_uppercase(),
            Self::Pascal => field
                .split('_')
                .map(|s| s[..1].to_ascii_uppercase() + &s[1..])
                .collect(),
            Self::Camel => {
                let pascal = Self::Pascal.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            Self::ScreamingSnake => Self::Upper.apply_to_field(field),
            Self::Kebab => field.replace('_', "-"),
            Self::ScreamingKebab => Self::ScreamingSnake
                .apply_to_field(field)
                .replace('_', "-"),
        }
    }

    pub(crate) fn apply_to_variant(self, variant: &str) -> String {
        match self {
            Self::Pascal => variant.to_string(),
            Self::Lower => variant.to_ascii_lowercase(),
            Self::Upper => variant.to_ascii_uppercase(),
            Self::Camel => variant[..1].to_ascii_lowercase() + &variant[1..],
            Self::Snake => variant
                .split(char::is_uppercase)
                .map(str::to_ascii_lowercase)
                .collect::<Vec<_>>().join("_"),
            Self::ScreamingSnake => Self::Snake
                .apply_to_variant(variant)
                .to_ascii_uppercase(),
            Self::Kebab => Self::Snake
                .apply_to_variant(variant)
                .replace('_', "-"),
            Self::ScreamingKebab => Self::ScreamingSnake
                .apply_to_variant(variant)
                .replace('_', "-"),
        }
    }
}
