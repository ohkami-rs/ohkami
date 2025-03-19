use std::{borrow::Cow, collections::VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RouteSegments {
    literal:  Cow<'static, str>,
    segments: VecDeque<RouteSegment>,
}
impl RouteSegments {
    pub(crate) fn from_literal(literal: impl Into<Cow<'static, str>>) -> Self {
        let literal: Cow<'static, str> = literal.into();

        if literal.is_empty() {panic!("found an empty route")}

        if literal == "/" {
            return Self { literal, segments: VecDeque::new() }
        }

        if !literal.starts_with('/') {panic!("routes must start with '/': `{literal}`")}
        if literal.ends_with('/') {panic!("routes must not ends with '/' except for \"/\": `{literal}`")}

        let mut segments   = VecDeque::new();
        let mut prev_slash = 0;
        for slash in literal
            .char_indices()
            .filter_map(|(i, ch)| (ch == '/').then_some(i))
            .skip(1)
            .chain(Some(literal.len()))
        {
            segments.push_back(RouteSegment::new(match &literal {
                Cow::Borrowed(s) => Cow::Borrowed(&s[prev_slash..slash]),
                Cow::Owned(s) => Cow::Owned((&s[prev_slash..slash]).to_owned())
            }).expect(&format!("invalid route `{literal}`")));

            prev_slash = slash;
        }

        Self { literal, segments }
    }

    pub(crate) fn literal(&self) -> &str {
        &self.literal
    }

    pub(crate) fn n_params(&self) -> usize {
        self.segments.iter()
            .filter(|s| matches!(s, RouteSegment::Param(_)))
            .count()
    }

    pub(crate) fn merged(self, another: Self) -> Self {
        let mut literal: Cow<'_, str> = Cow::Owned(format!(
            "{}/{}",
            self.literal().trim_end_matches('/'),
            another.literal().trim_start_matches('/')
        ));
        if literal != "/" && literal.ends_with('/') {
            let _ = literal.to_mut().pop();
        }

        let mut segments = self.segments;
        segments.extend(another);

        Self { literal, segments }
    }
}
impl std::ops::Deref for RouteSegments {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.literal
    }
}
impl std::fmt::Display for RouteSegments {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&**self)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum RouteSegment {
    Static(Cow<'static, str>),
    Param (Cow<'static, str>),
}
impl RouteSegment {
    pub(crate) fn new(segment: Cow<'static, str>) -> Result<Self, String> {
        fn validate_segment_name(mut name: impl DoubleEndedIterator<Item = char>) -> Result<(), String> {
            name.all(|c| matches!(
                c,
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '.' | '_' | '~'
            )).then_some(()).ok_or_else(|| format!(
                "path can only contain: [a-zA-Z0-9-\\._~]"
            ))
        }

        let mut segment_chars = segment.starts_with('/')
            .then_some(&segment[1..]).ok_or_else(|| "path segment must start with '/'")?
            .chars()
            .peekable();
        match segment_chars.peek() {
            None => Err(format!("Found an empty route segment")),
            Some(':') => {
                let _/* colon */ = segment_chars.next();
                let _/* validation */ = validate_segment_name(segment_chars)?;
                Ok(Self::Param(segment))
            },
            _ => {
                let _/* validation */ = validate_segment_name(segment_chars)?;
                Ok(Self::Static(segment))
            }
        }
    }
}
impl std::fmt::Debug for RouteSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Param (name) => f.write_str(name),
            Self::Static(s)    => f.write_str(s),
        }
    }
}

pub(crate) struct RouteSegmentsIterator(
    std::collections::vec_deque::IntoIter<RouteSegment>
);
impl Iterator for RouteSegmentsIterator {
    type Item = RouteSegment;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
impl IntoIterator for RouteSegments {
    type Item = RouteSegment;
    type IntoIter = RouteSegmentsIterator;
    fn into_iter(self) -> Self::IntoIter {
        RouteSegmentsIterator(self.segments.into_iter())
    }
}
