mod fang; pub use fang::*;
mod handler; pub use handler::*;

use std::{collections::VecDeque, str::Chars, iter::Peekable};
type Range = std::ops::Range<usize>;


#[derive(Clone)]
#[cfg_attr(any(test, debug_assertions), derive(Debug))]
pub struct RouteSections {
    route:    &'static [u8],
    sections: VecDeque<RouteSection>
} impl RouteSections {
    pub(crate) fn from_literal(route: &'static str) -> Self {
        if route.is_empty() {panic!("Found an empty route: `{route}`")}
        if !route.starts_with('/') {panic!("Routes must start with '/': `{route}`")}

        if route == "/" {return Self{ route: route.as_bytes(), sections: VecDeque::new()}}

        let mut sections = VecDeque::new();
        let mut section_start = 1; /* skip initial '/' */
        for section in {let mut s = route.split('/'); s.next(); s} {
            #[cfg(debug_assertions)] println!("section: '{section}' ('{}')",
                std::str::from_utf8(
                    &route.as_bytes()[section_start..(section_start + section.len())]
                ).unwrap()
            );

            let section = match RouteSection::new(route.as_bytes(), section_start..(section_start + section.len())) {
                Err(e) => panic!("{e}: `{route}`"),
                Ok(rs) => {section_start += section.len() + 1/* skip '/' */; rs}
            };
            sections.push_back(section)
        }
        Self{ sections, route:route.as_bytes() }
    }
} const _: () = {
    impl IntoIterator for RouteSections {
        type Item = <VecDeque<RouteSection> as IntoIterator>::Item;
        type IntoIter = <VecDeque<RouteSection> as IntoIterator>::IntoIter;
        fn into_iter(self) -> Self::IntoIter {self.sections.into_iter()}
    }
};

#[derive(Clone)]
pub enum RouteSection {
    Static{ route: &'static [u8], range: Range },
    Param,
} impl RouteSection {
    pub(crate) fn new(route: &'static [u8], range: Range) -> Result<Self, String> {
        let mut section_chars = std::str::from_utf8(&route[range.clone()]).unwrap().chars().peekable();

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
                Ok(Self::Static{ route, range })
            }
        }
    }
} const _: () = {
    #[cfg(any(test, debug_assertions))]
    impl std::fmt::Debug for RouteSection {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Param                   => f.write_str(":Param"),
                Self::Static { route, range } => f.write_str(std::str::from_utf8(&route[range.clone()]).unwrap()),
            }
        }
    }
};
