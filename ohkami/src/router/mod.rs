pub mod route; 
pub(crate) mod trie_tree;

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
    #[inline] pub(crate) fn search<'req>(
        &self,
        request_method: &'req str,
        request_path:   &'req str,
    ) -> Option<(
        &Vec<Fang<'router>>,
        &HandleFunc<'router>,
        PathParams<'req>,
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
            &Vec<Fang<'router>>,
            &HandleFunc<'router>,
            PathParams<'router>,
        )> {

        }
    }
};
