mod trie;
mod radix;

use std::{fmt::Debug};
use super::{trie as t, radix as r};
use crate::layer3_fang_handler::{Handler, Fang, FrontFang};

fn eq_as_set<Item: Clone + PartialEq>(left: &Vec<Item>, right: &Vec<Item>) -> bool {
    let len = left.len();
    if right.len() != len {return false}

    let mut right = right.clone();
    for item in left {
        let Some(pos) = right.iter().position(|i| i == item)
            else {return false};
        right.remove(pos);
    }
    right.is_empty()
}


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

    impl Debug for FrontFang {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_tuple("Front")
            .field(&self.id)
            .finish()
        }
    }


    impl PartialEq for Fang {
        fn eq(&self, other: &Self) -> bool {
            self.id() == other.id()
        }
    }

    impl PartialEq for Handler {
        fn eq(&self, _: &Self) -> bool {
            true
        }
    }

    impl PartialEq for FrontFang {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
};

#[cfg(test)] const _: () = {
    impl Debug for t::TrieRouter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Trie")
                .field("GET", &self.GET)
                .finish()
        }
    }

    impl Debug for t::Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match &self.pattern {
                None => f.debug_struct("Node")
                    .field("handler", &self.handler)
                    .field("fangs", &self.fangs)
                    .field("children", &self.children)
                    .finish(),
                Some(p) => f.debug_struct(&format!("Node({p:?})"))
                    .field("handler", &self.handler)
                    .field("fangs", &self.fangs)
                    .field("children", &self.children)
                    .finish(),
            }
        }
    }

    impl PartialEq for t::TrieRouter {
        fn eq(&self, other: &Self) -> bool {
            self.GET == other.GET
        }
    }

    impl PartialEq for t::Node {
        fn eq(&self, other: &Self) -> bool {
            self.pattern  == other.pattern     &&
            self.handler  == other.handler     &&
            eq_as_set(&self.fangs, &other.fangs) &&
            self.children == other.children
        }
    }
};

#[cfg(test)] const _: () = {
    impl Debug for r::RadixRouter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Radix")
                .field("GET", &self.GET)
                .finish()
        }
    }

    impl Debug for r::Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct(&format!("Node({:?})", &self.patterns))
                .field("handler", &self.handler)
                .field("front", &self.front)
                .field("children", &self.children)
                .finish()
        }
    }


    impl PartialEq for r::RadixRouter {
        fn eq(&self, other: &Self) -> bool {
            self.GET == other.GET
        }
    }

    impl PartialEq for r::Node {
        fn eq(&self, other: &Self) -> bool {
            self.patterns == other.patterns                        &&
            self.handler  == other.handler                         &&
            eq_as_set(&self.front.to_vec(), &other.front.to_vec()) &&
            self.children == other.children
        }
    }

    impl PartialEq for r::Pattern {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Param,          Self::Param)           => true,
                (Self::Static(self_s), Self::Static(other_s)) => self_s == other_s,
                _                                             => false,
            }
        }
    }


    impl Clone for r::Pattern {
        fn clone(&self) -> Self {
            match self {
                Self::Param     => Self::Param,
                Self::Static(s) => Self::Static(s)
            }
        }
    }
};
