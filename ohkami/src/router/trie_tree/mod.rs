mod node; pub(super) use node::TrieNode;
mod pattern; pub(super) use pattern::TriePattern;

use crate::{handler::Handler, fang::Fangs};

use super::Router;

#[allow(non_snake_case)]
pub(crate) struct TrieTree<'router> {
    GET: TrieNode<'router>,
    POST: TrieNode<'router>,
    PATCH: TrieNode<'router>,
    DELETE: TrieNode<'router>,
} impl<'router> TrieTree<'router> {
    pub(crate) fn new<const N: usize>(handlers: [Handler<'router>; N]) -> Self {
        let mut tree = Self {
            GET: TrieNode::root(),
            POST: TrieNode::root(),
            PATCH: TrieNode::root(),
            DELETE: TrieNode::root(),
        };
        for Handler { route, GET, POST, PATCH, DELETE } in handlers {
            if let Some(func) = GET {tree.GET.register(route, func)}
            if let Some(func) = POST {tree.POST.register(route, func)}
            if let Some(func) = PATCH {tree.PATCH.register(route, func)}
            if let Some(func) = DELETE {tree.DELETE.register(route, func)}
        }
        tree
    }
    pub(crate) fn apply(&mut self, fangs: Fangs<'router>) {
        compile_error!("TODO")
    }

    pub(crate) fn into_radix(self) -> Router<'router> {
        
    }
}
