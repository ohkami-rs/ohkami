use std::ops::Range;

pub(crate) enum TriePattern {
    Section { route_str: &'static str, range: Range<usize> },
    Param,
    Nil,
} impl TriePattern {
    
}
