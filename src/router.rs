use std::str::Split;

use crate::{
    server::Handler,
    components::method::Method,
};

#[allow(non_snake_case)]
pub(crate) struct Router<'p> {
    GET:    TrieTree<'p>,
    POST:   TrieTree<'p>,
    PATCH:  TrieTree<'p>,
    DELETE: TrieTree<'p>,
}
impl<'p> Router<'p> {
    pub fn new() -> Self {
        Self {
            GET:    TrieTree::new(),
            POST:   TrieTree::new(),
            PATCH:  TrieTree::new(),
            DELETE: TrieTree::new(),
        }
    }
    pub fn register(&mut self, method: Method, path_str: &'static str) {
        match method {
            Method::GET    => &mut self.GET,
            Method::POST   => &mut self.POST,
            Method::PATCH  => &mut self.PATCH,
            Method::DELETE => &mut self.DELETE,
        }.insert(
            path_str.split('/')
        )
    }
}

struct TrieTree<'p>(
    Node<'p>
); impl<'p> TrieTree<'p> {
    fn new() -> Self {
        Self(Node::new("/"))
    }
    fn insert(&mut self, path: Split<'p, char>) {
        
    }
}

struct Node<'p> {
    pattern:  &'p str,
    name:     Option<&'p str>,
    handler:  Option<Handler>,
    children: Vec<Node<'p>>,
} impl<'p> Node<'p> {
    fn new(pattern: &'p str) -> Self {
        Self {
            pattern,
            name:     None,
            handler:  None,
            children: Vec::new(),
        }
    }
}
