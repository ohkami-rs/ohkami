//! serde case specifiers
//!
//! based on https://github.com/serde-rs/serde/blob/930401b0dd58a809fce34da091b8aa3d6083cb33/serde_derive/src/internals/case.rs

#[derive(Clone, Copy)]
enum Case {
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
        match s {
            "lowercase"            => Self::Lower,
            "UPPERCASE"            => Self::Upper,
            "PascalCase"           => Self::Pascal,
            "camelCase"            => Self::Camel,
            "snake_case"           => Self::Snake,
            "SCREAMING_SNAKE_CASE" => Self::ScreamingSnake,
            "kebab-case"           => Self::Kebab,
            "SCREAMING-KEBAB-CASE" => Self::ScreamingKebab,
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
                .map(str::to_ascii_lowercase())
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
