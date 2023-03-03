use std::ops::Range;

pub(in super::super) enum TriePattern {
    Section { route_str: &'static str, range: Range<usize> },
    Param,
    Nil,
} impl TriePattern {
    
}
