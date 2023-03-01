use std::ops::Range;

pub(super) struct TrieTree {

}

pub(super) struct TrieNode {

}

pub(super) enum Pattern {
    Section { route_str: &'static str, range: Range<usize> },
    Param,
    Nil,
}
