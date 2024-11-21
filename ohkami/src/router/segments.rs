use std::collections::vec_deque::{IntoIter as VecDequeIterator, VecDeque};
use std::ops::Range;


#[derive(Clone, Debug)]
pub(crate) struct RouteSegments {
    literal: &'static str,
    segments: VecDequeIterator<RouteSegment>,
}
impl RouteSegments {
    pub(crate) fn from_literal(literal: &'static str) -> Self {
        if literal.is_empty() {panic!("Found an empty route")}
        if !literal.starts_with('/') {panic!("Routes must start with '/': `{literal}`")}

        if literal == "/" {
            return Self {
                literal,
                segments: VecDeque::new().into_iter()
            }
        }

        let mut segments = VecDeque::new();
        let mut i = 0;
        for segment in literal.split('/').skip(1) {
            let pos = i + 1/* '/' */ + segment.len();
            let segment = match RouteSegment::new(literal, i..pos) {
                Err(e) => panic!("{e}: `{literal}`"),
                Ok(rs) => rs,
            };
            segments.push_back(segment);
            i = pos;
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
    Static { route: &'static str, range: Range<usize> },
    Param  { name: &'static str },
}
impl RouteSegment {
    pub(crate) fn new(route: &'static str, range: Range<usize>) -> Result<Self, String> {
        fn validate_segment_name(mut name: impl DoubleEndedIterator<Item = char>) -> Result<(), String> {
            let is_invalid_head_or_tail_char = |c: char| !/* NOT */ matches!(c,
                '0'..='9' | 'a'..='z' | 'A'..='Z'
            );
            let is_invalid_char = |c: char| !/* NOT */ matches!(c,
                '.' | '-' | '_' | '0'..='9' | 'a'..='z' | 'A'..='Z'
            );

            let head = name.next().ok_or(format!("Found an empty segment name"))?;
            if is_invalid_head_or_tail_char(head) {
                return Err(format!("Path segment can't start with '{head}'"))
            }

            if let Some(tail) = name.next_back() {
                if is_invalid_head_or_tail_char(tail) {
                    return Err(format!("Path segment can't end with '{tail}'"))
                }
            }

            if name.any(is_invalid_char) {
                return Err(format!("Path segment can only contain '.' | '-' | '_' | '0'..='9' | 'a'..='z' | 'A'..='Z'"))
            }

            Ok(())
        }

        let mut segment_chars = route[range.start+1/* skip '/' */..range.end].chars().peekable();
        match segment_chars.peek() {
            None => Err(format!("Found an empty route segment")),
            Some(':') => {
                let _/* colon */ = segment_chars.next();
                let _/* validation */ = validate_segment_name(segment_chars)?;
                Ok(Self::Param { name: &route[range.start+1/* skip '/' */..range.end] })
            },
            _ => {
                let _/* validation */ = validate_segment_name(segment_chars)?;
                Ok(Self::Static { route, range })
            }
        }
    }
}
const _: () = {
    impl std::fmt::Debug for RouteSegment {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Param  { name }         => f.write_str(name),
                Self::Static { route, range } => f.write_str(&route[range.clone()]),
            }
        }
    }
};
