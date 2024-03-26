#![allow(non_snake_case)]

#[cfg(test)]
mod _test;

mod trie;
pub(crate) use trie::TrieRouter;

mod radix;
pub(crate) use radix::RadixRouter;

use std::{collections::VecDeque, iter::Peekable, str::Chars};


#[derive(Clone, Debug)]
pub struct RouteSections(
    VecDeque<RouteSection>
);
impl RouteSections {
    pub(crate) fn from_literal(route: &'static str) -> Self {
        if route.is_empty() {panic!("Found an empty route: `{route}`")}
        if !route.starts_with('/') {panic!("Routes must start with '/': `{route}`")}

        if route == "/" {return Self(VecDeque::new())}

        let mut sections = VecDeque::new();
        for section in {let mut s = route.split('/'); s.next(); s} {
            let section = match RouteSection::new(section.as_bytes()) {
                Err(e) => panic!("{e}: `{route}`"),
                Ok(rs) => rs,
            };
            sections.push_back(section)
        }
        Self(sections)
    }
}
const _: () = {
    impl IntoIterator for RouteSections {
        type Item = <VecDeque<RouteSection> as IntoIterator>::Item;
        type IntoIter = <VecDeque<RouteSection> as IntoIterator>::IntoIter;
        fn into_iter(self) -> Self::IntoIter {self.0.into_iter()}
    }
};

#[derive(Clone)]
pub enum RouteSection {
    Static(&'static [u8]),
    Param,
}
impl RouteSection {
    pub(crate) fn new(section_bytes: &'static [u8]) -> Result<Self, String> {
        let mut section_chars = std::str::from_utf8(section_bytes).unwrap().chars().peekable();

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
                Ok(Self::Static(section_bytes))
            }
        }
    }
}
const _: () = {
    impl std::fmt::Debug for RouteSection {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Param         => f.write_str(":Param"),
                Self::Static(bytes) => f.write_str(std::str::from_utf8(bytes).unwrap()),
            }
        }
    }
};
