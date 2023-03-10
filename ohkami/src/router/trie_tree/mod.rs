pub(crate) mod node; pub(crate) use node::TrieNode;
pub(crate) mod pattern; pub(crate) use pattern::TriePattern;

use crate::{handler::Handlers, fang::Fangs};
use super::Router;


#[allow(non_snake_case)]
pub(crate) struct TrieTree {
    GET: TrieNode,
    POST: TrieNode,
    PATCH: TrieNode,
    DELETE: TrieNode,
} impl TrieTree {
    pub(crate) fn new<const N: usize>(handlers: [Handlers; N]) -> Self {
        let mut tree = Self {
            GET: TrieNode::root(),
            POST: TrieNode::root(),
            PATCH: TrieNode::root(),
            DELETE: TrieNode::root(),
        };
        for Handlers { route, GET, POST, PATCH, DELETE } in handlers {
            if let Some(func) = GET {tree.GET.register(route, func)}
            if let Some(func) = POST {tree.POST.register(route, func)}
            if let Some(func) = PATCH {tree.PATCH.register(route, func)}
            if let Some(func) = DELETE {tree.DELETE.register(route, func)}
        }
        tree
    }
    pub(crate) fn apply(&mut self, fangs: Fangs) {
        // abc order
        self.DELETE.apply(fangs.clone());
        self.GET.apply(fangs.clone());
        self.PATCH.apply(fangs.clone());
        self.POST.apply(fangs);
    }
    pub(crate) fn into_radix(self) -> Router {
        Router {
            GET: self.GET.into_radix(),
            POST: self.POST.into_radix(),
            PATCH: self.PATCH.into_radix(),
            DELETE: self.DELETE.into_radix(),
        }
    }
}
