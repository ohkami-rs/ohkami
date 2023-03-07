use std::ops::Range;
use crate::router::Pattern;


pub(crate) enum TriePattern {
    Section { route_str: &'static str, range: Range<usize> },
    Param,
    Nil,
} impl TriePattern {
    pub(crate) fn parse(range: Range<usize>, route_str: &str) -> Self {
        let section = &route_str[range];
        if section.starts_with(':') {
            let param_name = &section[1..];
            if let Err(msg) = validate_section(param_name) {
                panic!("path parameter `{param_name}` in route `{route_str}` is invalid: \"{msg}\"");
            }
            Self::Param
        } else {
            if let Err(msg) = validate_section(section) {
                panic!("section `{section}` in route `{route_str}` is invalid: \"{msg}\"");
            }
            Self::Section { route_str, range }
        }
    }
    pub(super) fn into_radix(self) -> Pattern {
        match self {
            Self::Nil => Pattern::Nil,
            Self::Param => Pattern::Param,
            Self::Section { route_str, range } => Pattern::Str(&route_str[range]),
        }
    }

    pub(crate) fn is_section(&self) -> bool {
        match self {
            Self::Section { .. } => true,
            _ => false,
        }
    }
    pub(crate) fn is_nil(&self) -> bool {
        match self {
            Self::Nil => true,
            _ => false,
        }
    }
    pub(crate) fn get_section(&self) -> Option<(/*route_str*/&'static str, /*range*/&Range<usize>)> {
        match self {
            Self::Section { route_str, range } => Some((route_str, range)),
            _ => None,
        }
    }
    pub(crate) fn get_section_mut(&mut self) -> Option<(/*route_str*/&'static str, /*range*/&mut Range<usize>)> {
        match self {
            Self::Section { route_str, range } => Some((route_str, range)),
            _ => None,
        }
    }
    pub(super) fn merge_sections(&mut self, child_pattern: Self) {
        let Some((this_route, this_range)) = self.get_section_mut() else {return};
        let Some((child_route, child_range)) = child_pattern.get_section() else {return};

        if this_route == child_route
        && this_range.end == child_range.start {
            this_range.end = child_range.end
        }
    }
}

pub(crate) fn validate_section(section: &str) -> Result<(), &'static str> {
    match section.len() {
        0 => Err("empty string"),
        1 => match section.chars().next().unwrap() {
            'a'..='z' | 'A'..='Z' => Ok(()),
            _ => Err("route section or path param's name must starts with 'a'..='z' | 'A'..='Z'"),
        },
        _ => {
            let chars = section.chars();
            match chars.next().unwrap() {
                'a'..='z' | 'A'..='Z' => (),
                _ => return Err("route section or path param's name must start with 'a'..='z' | 'A'..='Z'"),
            };
            for ch in chars {
                match ch {
                    'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => (),
                    _ => return Err("route section or path param's name must consist of 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'"),
                }
            }
            Ok(())
        },
    }
}


const _: () = {
    impl Clone for TriePattern {
        fn clone(&self) -> Self {
            match self {
                Self::Nil => Self::Nil,
                Self::Param => Self::Param,
                Self::Section { route_str, range } => Self::Section { route_str, range: range.clone() }
            }
        }
    }
};
