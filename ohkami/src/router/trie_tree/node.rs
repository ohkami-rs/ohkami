use super::pattern::TriePattern::{self, *};
use crate::{
    router::Node,
    fang::{Fang, Fangs, route::{FangsRoute, FangRoutePattern}},
    handler::{Handler, route::HandlerRoute},
};


pub(crate) struct TrieNode {
    pattern:     TriePattern,
    fangs:       Vec<Fang>,
    handler:     Option<Handler>,
    children:    Vec<TrieNode>,
} impl TrieNode {
    pub(super) fn root() -> Self {
        Self::new(
            TriePattern::Nil
        )
    }
    pub(super) fn register(&mut self, mut route: HandlerRoute, handler: Handler) {
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
    pub(super) fn apply(&mut self, fangs: Fangs) {
        for (route, vec_fang) in fangs {
            self.register_fangs(route, vec_fang)
        }
    }
    pub(super) fn into_radix(mut self) -> Node {
        let self_fangs: Vec<Fang> = {
            let mut v = Vec::with_capacity(self.fangs.len());
            for fang in &self.fangs {
                let fang: Fang = Box::new(move |c, request| Box::pin(
                    fang(c, request)
                ));
                v.push(fang)
            }
            v
        };

        let mut sections = vec![(self.pattern.clone(), self_fangs)];
        (self, sections) = Self::merge_single_child(self, sections);

        Node {
            handler:  self.handler,
            children: Box::leak(self.children.into_iter().map(|c| c.into_radix()).collect()),
            sections:
                Box::leak(sections
                    .into_iter()
                    .map(|(pat, fangs)| super::super::Section {
                        pattern: pat.into_radix(),
                        fangs:   fangs.leak(),
                    })
                    .collect()
                ),
        }
    }
} const _: () = {
    impl TrieNode {
        fn new(pattern: TriePattern) -> Self {
            Self {
                pattern,
                fangs :   vec![],
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

        fn register_fangs(&mut self, mut route: FangsRoute, fangs: Vec<Fang>) {
            if let Some(next_pattern) = route.next() {

                let pattern = match next_pattern {
                    FangRoutePattern::AnyAfter => {
                        self.fangs = fangs;
                        return
                    }
                    section_or_param => section_or_param.into_trie()
                };

                if let Some(child) = self.matchablle_child_mut(&pattern) {
                    child.register_fangs(route, fangs)
                } else {
                    let mut child = TrieNode::new(pattern);
                    child.register_fangs(route, fangs);
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
            mut sections: Vec<(TriePattern, Vec<Fang>)>,
        ) -> (
            Self,
            Vec<(TriePattern, Vec<Fang>)>,
        ) {
            let (this_pattern, this_fangs) = &mut sections.last_mut().unwrap();

            if self.children.len() == 1
            && self.handler.is_none() {

                let child = self.children.pop().unwrap();
                let (child_pattern, child_fangs) = (child.pattern.clone(), child.fangs);

                if this_pattern.is_section() && child_pattern.is_section() {
                    this_pattern.merge_sections(child_pattern);
                    this_fangs.extend(child_fangs);
                } else if this_pattern.is_nil() {
                    *this_pattern = child_pattern
                } else {
                    sections.push((child_pattern, child_fangs))
                }

                self.children = child.children;
                self.handler = child.handler;
                Self::merge_single_child(self, sections)

            } else {
                (self, sections)
            }
        }
    }
};
