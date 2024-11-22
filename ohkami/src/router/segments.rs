use std::collections::vec_deque::{IntoIter as VecDequeIterator, VecDeque};


#[derive(Clone, Debug)]
pub(crate) struct RouteSegments {
    literal: &'static str,
    segments: VecDequeIterator<RouteSegment>,
}
impl RouteSegments {
    pub(crate) fn from_literal(literal: &'static str) -> Self {        
        if literal.is_empty() {panic!("found an empty route")}

        if literal == "/" {
            return Self {
                literal,
                segments: VecDeque::new().into_iter()
            }
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
            segments.push_back(RouteSegment::new(&literal[prev_slash..slash])
                .expect(&format!("invalid route `{literal}`")));
            prev_slash = slash;
        }

        Self { literal, segments:segments.into_iter() }
    }

    pub(crate)  fn literal(&self) -> &'static str {
        self.literal
    }
}
const _: () = {
    impl Iterator for RouteSegments {
        type Item = RouteSegment;
        fn next(&mut self) -> Option<Self::Item> {
            self.segments.next()
        }
    }
};

#[derive(Clone)]
pub(crate) enum RouteSegment {
    Static(&'static str),
    Param (&'static str),
}
impl RouteSegment {
    pub(crate) fn new(segment: &'static str) -> Result<Self, String> {
        fn validate_segment_name(mut name: impl DoubleEndedIterator<Item = char>) -> Result<(), String> {
            let is_invalid_head_or_tail_char = |c: char| !/* NOT */ matches!(c,
                '0'..='9' | 'a'..='z' | 'A'..='Z'
            );
            let is_invalid_char = |c: char| !/* NOT */ matches!(c,
                '.' | '-' | '_' | '0'..='9' | 'a'..='z' | 'A'..='Z'
            );

            let head = name.next().ok_or(format!("found an empty segment name"))?;
            if is_invalid_head_or_tail_char(head) {
                return Err(format!("path segment can't start with '{head}'"))
            }

            if let Some(tail) = name.next_back() {
                if is_invalid_head_or_tail_char(tail) {
                    return Err(format!("path segment can't end with '{tail}'"))
                }
            }

            if name.any(is_invalid_char) {
                return Err(format!("path segment can only contain '.' | '-' | '_' | '0'..='9' | 'a'..='z' | 'A'..='Z'"))
            }

            Ok(())
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
const _: () = {
    impl std::fmt::Debug for RouteSegment {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Param (name) => f.write_str(name),
                Self::Static(s)    => f.write_str(s),
            }
        }
    }
};
