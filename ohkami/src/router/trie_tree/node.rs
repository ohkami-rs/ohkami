use crate::{fang::Fang, handler::{HandleFunc, HandleRoute}};
use super::pattern::TriePattern::{self, *};

pub(crate) struct TrieNode<'router> {
    pattern:     TriePattern,
    fangs:       Vec<Fang<'router>>,
    handle_func: Option<HandleFunc<'router>>,
    children:    Vec<TrieNode<'router>>,
} impl<'router> TrieNode<'router> {
    pub(super) fn root() -> Self {
        Self {
            pattern:     Nil,
            fangs:       vec![],
            handle_func: None,
            children:    vec![],
        }
    }
    pub(super) fn register(&mut self, route: HandleRoute, handle_func: HandleFunc<'router>) {
        
    }
}
