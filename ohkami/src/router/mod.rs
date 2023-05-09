mod trie_tree;
mod radix_tree;

use async_std::net::TcpStream;
use crate::{Fang, handler::Handler, Context, Request};


/*===== definitions =====*/
pub(crate) struct Router {
    tree:  Tree,
    procs: Procs,
}

struct Tree {
    GET:    Node,
    POST:   Node,
    PATCH:  Node,
    DELETE: Node,
}
struct Node {
    id: usize,
    children: &'static [Node],
}

struct Procs {
    GET:    &'static [(&'static [Fang], Handler)],
    POST:   &'static [(&'static [Fang], Handler)],
    PATCH:  &'static [(&'static [Fang], Handler)],
    DELETE: &'static [(&'static [Fang], Handler)],
}


/*===== impls =====*/
impl Router {
    pub(crate) fn handler(
        &self,
        c: Context,
        stream:  TcpStream,
        request: Request,
    ) {
        todo!()
    }

    fn search(&self, path: &str) -> Option</* id */usize> {
        todo!()
    }
}
