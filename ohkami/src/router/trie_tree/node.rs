use super::pattern::TriePattern::{self, *};
use crate::{
    router::Node,
    fang::{Fang, Fangs},
    handler::{Handler, route::HandleRoute},
};


pub(crate) struct TrieNode<'router> {
    pattern:     TriePattern,
    fangs:       Vec<Fang<'router>>,
    handle_func: Option<Handler<'router>>,
    children:    Vec<TrieNode<'router>>,
} impl<'req> TrieNode<'req> {
    pub(super) fn root() -> Self {
        Self {
            pattern:     Nil,
            fangs:       vec![],
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
    pub(super) fn into_radix(self) -> Node<'req> {
        todo!()
    }
}
