use super::pattern::TriePattern::{self, *};
use crate::{
    router::Node,
    fang::{Fang, Fangs, combine, combine_optional},
    handler::{Handler, route::HandleRoute},
};


pub(crate) struct TrieNode<'router> {
    pattern:     TriePattern,
    fang:        Option<Fang<'router>>,
    handle_func: Option<Handler<'router>>,
    children:    Vec<TrieNode<'router>>,
} impl<'req> TrieNode<'req> {
    pub(super) fn root() -> Self {
        Self {
            pattern:     Nil,
            fang :       None,
            handle_func: None,
            children:    vec![],
        }
    }
    pub(super) fn register(&mut self, route: HandleRoute, handle_func: Handler<'req>) {
        compile_error!("TODO")
    }
    pub(super) fn apply(&mut self, fangs: Fangs<'req>) {
        compile_error!("TODO")
    }
    pub(super) fn into_radix(mut self) -> Node<'req> {
        let mut patterns = vec![(self.pattern.clone(), self.fang)];
        (self, patterns) = Self::merge_single_child(self, patterns);

        Node {
            patterns: Box::leak(patterns.into_iter().map(|(pat, fang)| (pat.into_radix(), fang)).collect()),
            handle_func: self.handle_func,
            children: Box::leak(self.children.into_iter().map(|c| c.into_radix()).collect()),
        }
    }
} const _: () = {
    impl<'req> TrieNode<'req> {
        fn merge_single_child(
            mut self,
            mut patterns: Vec<(TriePattern, Option<Fang<'req>>)>,
        ) -> (
            Self,
            Vec<(TriePattern, Option<Fang<'req>>)>,
        ) {
            let (this_pattern, this_fang) = &mut patterns.last_mut().unwrap();

            if self.children.len() == 1
            && self.handle_func.is_none() {

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
                self.handle_func = child.handle_func;
                Self::merge_single_child(self, patterns)

            } else {

                (self, patterns)

            }
        }
    }
};
