mod trie;

use std::{fmt::Debug};
use super::{trie as t, radix as r};
use crate::layer3_fang_handler::{Fang, Handler};

#[cfg(test)] const _: () = {
    impl Debug for Fang {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple(match self {
                Self::Front(_) => "Front",
            })
                .field(self.id())
                .finish()
        }
    }

    impl Debug for Handler {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Handler")
        }
    }

    impl PartialEq for Fang {
        fn eq(&self, other: &Self) -> bool {
            self.id() == other.id()
        }
    }

    impl PartialEq for Handler {
        fn eq(&self, other: &Self) -> bool {
            true
        }
    }
};

#[cfg(test)] const _: () = {
    impl Debug for t::TrieRouter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Trie")
                .field("GET", &self.GET)
                .field("POST", &self.POST)
                .finish()
        }
    }

    impl Debug for t::Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.pattern {
                None => f.debug_struct("Node")
                    .field("fangs", &self.fangs)
                    .field("handler", &self.handler)
                    .field("children", &self.children)
                    .finish(),
                Some(p) => f.debug_struct("Node")
                    .field("fangs", &self.fangs)
                    .field("handler", &self.handler)
                    .field("children", &self.children)
                    .field("pattern", p)
                    .finish(),
            }
        }
    }

    impl PartialEq for t::TrieRouter {
        fn eq(&self, other: &Self) -> bool {
            self.GET == other.GET &&
            self.POST == other.POST
        }
    }

    impl PartialEq for t::Node {
        fn eq(&self, other: &Self) -> bool {
            self.pattern == other.pattern &&
            self.handler == other.handler &&
            self.fangs == other.fangs &&
            self.children == other.children
        }
    }

    impl PartialEq for t::Pattern {
        fn eq(&self, other: &Self) -> bool {
            match self {
                Self::Param => match other {
                    Self::Param => true,
                    _ => false,
                }
                Self::Static{ route, range } => {
                    let bytes = &route[range.clone()];
                    match other {
                        Self::Static{ route, range } => &route[range.clone()] == bytes,
                        _ => false
                    }
                }
            }
        }
    }
};
