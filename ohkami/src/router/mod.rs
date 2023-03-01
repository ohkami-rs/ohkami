#![allow(non_snake_case)]
mod trie_tree; use std::ops::Range;

use trie_tree::{TrieTree, TrieNode};

use crate::{
    handler::{HandleFunc, Handler},
    request::{REQUEST_BUFFER_SIZE, Request, PATH_PARAMS_LIMIT}, fang::Fang,
};


pub(crate) struct Router {
    GET: Node,
    POST: Node,
    PATCH: Node,
    DELETE: Node,
} impl Router {
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
        request_method: &'req str,
        request_path:   &'req str,
    ) -> Option<(
        &HandleFunc,
        &&'static [Fang],
        [Option<Range<usize>>; PATH_PARAMS_LIMIT],
    )> {
        match request_method {
            "GET" => self.GET.search(request_path),
            "POST" => self.POST.search(request_path),
            "PATCH" => self.PATCH.search(request_path),
            "DELETE" => self.DELETE.search(request_path),
            _ => return None
        }
    }
}

struct Node {
    patterns:    &'static [Pattern],
    fangs:       &'static [Fang],
    handle_func: Option<HandleFunc>,
    children:    &'static [Node],
} impl Node {
    fn radixized(trie_node: TrieNode) -> Self {

    }
}

enum Pattern {
    Str(&'static str),
    Param,
}

