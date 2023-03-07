use std::{collections::VecDeque, ops::Range};
use crate::router::trie_tree::{pattern::validate_section, TriePattern};


#[derive(PartialEq, Eq, Hash, Clone)]
pub struct FangsRoute(
    VecDeque<FangRoutePattern>
);
#[derive(PartialEq, Eq, Hash, Clone)]
pub(crate) enum FangRoutePattern {
    Section {range: Range<usize>, route_str: &'static str},
    Param,
    AnyAfter,
} impl FangRoutePattern {
    fn parse_section(range: Range<usize>, route_str: &'static str) -> Self {
        let section = &route_str[range];
        if section.starts_with(':') {
            let param_name = &section[1..];
            if let Err(msg) = validate_section(param_name) {
                panic!("path parameter `{param_name}` in route `{route_str}` is invalid: \"{msg}\"");
            }
            Self::Param
        } else {
            if let Err(msg) = validate_section(section) {
                tracing::error!("section `{section}` in route `{route_str}` is invalid: \"{msg}\"");
                panic!()
            }
            Self::Section { route_str, range }
        }
    }

    pub(crate) fn into_trie(self) -> TriePattern {
        match self {
            Self::AnyAfter => unreachable!(),
            Self::Param => TriePattern::Param,
            Self::Section { range, route_str } => TriePattern::Section { route_str, range },
        }
    }
}

impl FangsRoute {
    pub(super) fn parse(route_str: &'static str) -> Self {
        if !route_str.ends_with("/*") {
            panic!("Fang route `route_str` doesn't end with `/*`. Fang route must end with `/*`.")
        }
        let mut patterns = VecDeque::new();
        let mut pos = 1;
        for len in route_str[..route_str.len()-2].split('/').map(|s| s.len()) {
            patterns.push_back(FangRoutePattern::parse_section(pos..pos+len, route_str));
            pos += len + 1
        }
        Self({
            patterns.push_back(FangRoutePattern::AnyAfter);
            patterns
        })
    }
}


const _: (/* FangsRoute impls */) = {
    impl Iterator for FangsRoute {
        type Item = FangRoutePattern;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }
};
