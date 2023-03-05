use std::{ops::Range, collections::VecDeque};
use crate::handler::{Handler, HandleFunc};


pub trait Route: Sized {
    fn into_handle_route(self) -> HandleRoute;
    fn into_fang_route(self)   -> FangRoute;

    fn GET<'router>(self, handle_func: HandleFunc<'router>) -> Handler<'router> {
        Handler {
            route: self.into_handle_route(),
            GET: Some(handle_func),
            POST: None,
            PATCH: None,
            DELETE: None,
        }
    }
    fn POST<'router>(self, handle_func: HandleFunc<'router>) -> Handler<'router> {
        Handler {
            route: self.into_handle_route(),
            GET: None,
            POST: Some(handle_func),
            PATCH: None,
            DELETE: None,
        }
    }
    fn PATCH<'router>(self, handle_func: HandleFunc<'router>) -> Handler<'router> {
        Handler {
            route: self.into_handle_route(),
            GET: None,
            POST: None,
            PATCH: Some(handle_func),
            DELETE: None,
        }
    }
    fn DELETE<'router>(self, handle_func: HandleFunc<'router>) -> Handler<'router> {
        Handler {
            route: self.into_handle_route(),
            GET: None,
            POST: None,
            PATCH: None,
            DELETE: Some(handle_func),
        }
    }
}

pub struct HandleRoute(
    VecDeque<super::trie_tree::TriePattern>
);
pub struct FangRoute(
    VecDeque<FangRoutePattern>
); pub(super) enum FangRoutePattern {
    Section {route_str: &'static str, range: Range<usize>},
    Param,
    AnyAfter,
}


impl Route for &'static str {
    fn into_handle_route(self) -> HandleRoute {
        
    }

    fn into_fang_route(self) -> FangRoute {
        
    }
}


const _: (/* HandleRoute impls */) = {
    impl Iterator for HandleRoute {
        type Item = super::trie_tree::TriePattern;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }
};
const _: (/* FangRoute impls */) = {
    impl Iterator for FangRoute {
        type Item = FangRoutePattern;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }
};
