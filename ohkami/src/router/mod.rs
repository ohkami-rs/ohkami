#![allow(non_snake_case)]
mod trie_tree;

use trie_tree::{TrieTree, TrieNode};
use crate::{
    fang::Fang,
    request::PathParams,
    handler::{HandleFunc, Handler},
};


pub(crate) struct Router<'router> {
    GET: Node<'router>,
    POST: Node<'router>,
    PATCH: Node<'router>,
    DELETE: Node<'router>,
} impl<'router> Router<'router> {
    pub(crate) fn new<const N: usize>(handlers: [Handler; N]) -> Self {
        let mut trie_tree = TrieTree::new();
        for handler in handlers {
            trie_tree.register(handler)
        }
        Self {
            GET: Node::radixized(trie_tree.GET),
            POST: Node::radixized(trie_tree.POST),
            PATCH: Node::radixized(trie_tree.PATCH),
            DELETE: Node::radixized(trie_tree.DELETE),
        }
    }

    #[inline] pub(crate) fn search<'req>(
        &self,
        request_method: Method,
        request_path:   &'req str,
    ) -> Option<(
        &HandleFunc,
        &&'static [Fang],
        PathParams,
    )> {
        match request_method {
            "GET" => self.GET.search(request_path, PathParams::new()),
            "POST" => self.POST.search(request_path, PathParams::new()),
            "PATCH" => self.PATCH.search(request_path, PathParams::new()),
            "DELETE" => self.DELETE.search(request_path, PathParams::new()),
            _ => return None
        }
    }
}

struct Node<'router> {
    patterns:    &'static [Pattern],
    fangs:       Vec<Fang<'router>>,
    handle_func: Option<HandleFunc<'router>>,
    children:    Vec<Node<'router>>,
} impl<'router> Node<'router> {
    fn radixized(trie_node: TrieNode) -> Self {

    }
}

enum Pattern {
    Str(&'static str),
    Param,
}


const _: () = {
    impl<'router> Node<'router> {
        fn search<'req>(
            &self,
            mut request_path: &'req str,
            mut path_params:  PathParams,
        ) -> Option<(
            &HandleFunc<'router>,
            &&'static [Fang],
            PathParams,
        )> {
            
        }
    }
};
