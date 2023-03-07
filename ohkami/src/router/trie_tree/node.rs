use super::pattern::TriePattern::{self, *};
use crate::{
    router::Node,
    fang::{Fang, Fangs, combine_optional, route::{FangsRoute, FangRoutePattern}},
    handler::{Handler, route::HandlerRoute},
};


pub(crate) struct TrieNode<'router> {
    pattern:     TriePattern,
    fang:        Option<Fang<'router>>,
    handler:     Option<Handler<'router>>,
    children:    Vec<TrieNode<'router>>,
} impl<'req> TrieNode<'req> {
    pub(super) fn root() -> Self {
        Self::new(
            TriePattern::Nil
        )
    }
    pub(super) fn register(&mut self, mut route: HandlerRoute, handler: Handler<'req>) {
        if let Some(next_pattern) = route.next() {
            if let Some(child) = self.matchablle_child_mut(&next_pattern) {
                child.register(route, handler)
            } else {
                let mut child = TrieNode::new(next_pattern);
                child.register(route, handler);
                self.children.push(child)
            }
        } else {
            self.handler.replace(handler);
        }
    }
    pub(super) fn apply(&mut self, fangs: Fangs<'req>) {
        for (route, fang) in fangs {
            self.register_fang(route, fang)
        }
    }
    pub(super) fn into_radix(mut self) -> Node<'req> {
        let mut patterns = vec![(self.pattern.clone(), self.fang)];
        (self, patterns) = Self::merge_single_child(self, patterns);

        Node {
            patterns: Box::leak(patterns.into_iter().map(|(pat, fang)| (pat.into_radix(), fang)).collect()),
            handler:  self.handler,
            children: Box::leak(self.children.into_iter().map(|c| c.into_radix()).collect()),
        }
    }
} const _: () = {
    impl<'req> TrieNode<'req> {
        fn new(pattern: TriePattern) -> Self {
            Self {
                pattern,
                fang :    None,
                handler:  None,
                children: vec![],
            }
        }

        fn matchablle_child_mut(&mut self, pattern: &TriePattern) -> Option<&mut TrieNode> {
            for child in &mut self.children {
                if child.pattern.matches(pattern) {
                    return Some(child)
                }
            }
            None
        }

        fn register_fang(&mut self, route: FangsRoute, fang: Fang<'req>) {
            if let Some(next_pattern) = route.next() {

                let pattern = match next_pattern {
                    FangRoutePattern::AnyAfter => {
                        self.fang = combine_optional(self.fang.as_ref(), Some(&fang));
                        return
                    }
                    section_or_param => section_or_param.into_trie()
                };

                if let Some(child) = self.matchablle_child_mut(&pattern) {
                    child.register_fang(route, fang)
                } else {
                    let mut child = TrieNode::new(pattern);
                    child.register_fang(route, fang);
                    self.children.push(child)
                }

            } else {

                unreachable!("
                    - `FangsRoute`s have to end with an `_::AnyAfter` variant
                    - `regisiter_fang` returns when it found `_::AnyAfter`    
                ")

            }
        }

        fn merge_single_child(
            mut self,
            mut patterns: Vec<(TriePattern, Option<Fang<'req>>)>,
        ) -> (
            Self,
            Vec<(TriePattern, Option<Fang<'req>>)>,
        ) {
            let (this_pattern, this_fang) = &mut patterns.last_mut().unwrap();

            if self.children.len() == 1
            && self.handler.is_none() {

                let child = self.children.pop().unwrap();
                let (child_pattern, child_fang) = (child.pattern.clone(), child.fang);

                if this_pattern.is_section() && child_pattern.is_section() {
                    this_pattern.merge_sections(child_pattern);

                    *this_fang = combine_optional(this_fang.as_ref(), child_fang.as_ref());
                } else if this_pattern.is_nil() {
                    *this_pattern = child_pattern
                } else {
                    patterns.push((child_pattern, child_fang))
                }

                self.children = child.children;
                self.handler = child.handler;
                Self::merge_single_child(self, patterns)

            } else {

                (self, patterns)

            }
        }
    }
};
