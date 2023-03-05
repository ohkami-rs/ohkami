#![allow(non_snake_case)]
pub mod into_handlefunc;
pub mod route;

use std::{pin::Pin, future::Future, collections::VecDeque};
use crate::{
    context::Context,
    router::trie_tree::TriePattern,
    request::{Request, PathParams},
};

pub struct Handler<'router> {
    pub(crate) route:  HandleRoute,
    pub(crate) GET:    Option<HandleFunc<'router>>,
    pub(crate) POST:   Option<HandleFunc<'router>>,
    pub(crate) PATCH:  Option<HandleFunc<'router>>,
    pub(crate) DELETE: Option<HandleFunc<'router>>,
}
pub(crate) type HandleFunc<'router> =
    Box<dyn
        Fn(Context, Request<'router>, PathParams<'router>) -> Pin<
            Box<dyn
                Future<Output = ()>
                + Send
            >
        > + Send + Sync
    >
;
pub struct HandleRoute(
    VecDeque<TriePattern>
); const _: (/* HandleRoute impls */) = {
    impl Iterator for HandleRoute {
        type Item = TriePattern;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }
};

impl<'router> Handler<'router> {
    pub fn GET(mut self, handle_func: HandleFunc<'router>) -> Self {
        self.GET.replace(handle_func);
        self
    }
    pub fn POST(mut self, handle_func: HandleFunc<'router>) -> Self {
        self.POST.replace(handle_func);
        self
    }
    pub fn PATCH(mut self, handle_func: HandleFunc<'router>) -> Self {
        self.PATCH.replace(handle_func);
        self
    }
    pub fn DELETE(mut self, handle_func: HandleFunc<'router>) -> Self {
        self.DELETE.replace(handle_func);
        self
    }
}
