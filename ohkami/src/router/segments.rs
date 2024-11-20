use std::collections::vec_deque::{IntoIter as VecDequeIterator, VecDeque};
use std::iter::Peekable;
use std::ops::Range;
use std::str::Chars;


#[derive(Clone, Debug)]
pub(crate) struct RouteSegments {
    literal: &'static str,
    sections: VecDequeIterator<RouteSegment>,
}
impl RouteSegments {
    pub(crate) fn from_literal(literal: &'static str) -> Self {
        if literal.is_empty() {panic!("Found an empty route")}
        if !literal.starts_with('/') {panic!("Routes must start with '/': `{literal}`")}

        if literal == "/" {
            return Self {
                literal,
                sections: VecDeque::new().into_iter()
            }
        }

        let mut sections = VecDeque::new();
        let mut i = 0;
        for section in literal.split('/').skip(1) {
            let pos = i + 1/* '/' */ + section.len();
            let section = match RouteSegment::new(literal, i..pos) {
                Err(e) => panic!("{e}: `{literal}`"),
                Ok(rs) => rs,
            };
            sections.push_back(section);
            i = pos;
        }

        Self { literal, sections:sections.into_iter() }
    }

    pub(crate)  fn literal(&self) -> &'static str {
        self.literal
    }
}
const _: () = {
    impl Iterator for RouteSegments {
        type Item = RouteSegment;
        fn next(&mut self) -> Option<Self::Item> {
            self.sections.next()
        }
    }
};

#[derive(Clone)]
pub(super) enum RouteSegment {
    Static { route: &'static str, range: Range<usize> },
    Param,
}
impl RouteSegment {
    pub(crate) fn new(route: &'static str, range: Range<usize>) -> Result<Self, String> {
        let mut section_chars = route[range.clone()].chars().peekable();

        fn validate_section_name(mut name: Peekable<Chars>) -> Result<(), String> {
            let is_invalid_head_or_tail_char = |c: char| !/* NOT */ matches!(c,
                '0'..='9' | 'a'..='z' | 'A'..='Z'
            );

            let is_invalid_char = |c: char| !/* NOT */ matches!(c,
                '.' | '-' | '_' | '0'..='9' | 'a'..='z' | 'A'..='Z'
            );

            let Some(head) = name.next() else {return Err(format!("Found an empty section name"))};
            if is_invalid_head_or_tail_char(head) {
                return Err(format!("Path section can't start with '{head}'"))
            }

            let Some(tail) = name.next_back() else {return Ok(())};
            if is_invalid_head_or_tail_char(tail) {
                return Err(format!("Path section can't end with '{tail}'"))
            }

            for c in name {
                if is_invalid_char(c) {
                    return Err(format!("Path section can't contain '{c}'"))
                }
            }

            Ok(())
        }

        match section_chars.peek() {
            None => Err(format!("Found an empty route section_chars")),
            Some(':') => {
                let _/* colon */ = section_chars.next();
                let _/* validation */ = validate_section_name(section_chars)?;
                Ok(Self::Param)
            },
            _ => {
                let _/* validation */ = validate_section_name(section_chars)?;
                Ok(Self::Static { route, range })
            }
        }
    }
}
const _: () = {
    impl std::fmt::Debug for RouteSegment {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Param                   => f.write_str(":Param"),
                Self::Static { route, range } => f.write_str(&route[range.clone()]),
            }
        }
    }
};
