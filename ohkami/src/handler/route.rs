use std::collections::VecDeque;

use crate::router::trie_tree::TriePattern;

use super::{HandleRoute, HandleFunc, Handler};


pub trait Route: Sized {
    fn into_handle_route(self) -> HandleRoute;

    fn GET<'req>(self, handle_func: HandleFunc<'req>) -> Handler<'req> {
        Handler {
            route: self.into_handle_route(),
            GET: Some(handle_func),
            POST: None,
            PATCH: None,
            DELETE: None,
        }
    }
    fn POST<'req>(self, handle_func: HandleFunc<'req>) -> Handler<'req> {
        Handler {
            route: self.into_handle_route(),
            GET: None,
            POST: Some(handle_func),
            PATCH: None,
            DELETE: None,
        }
    }
    fn PATCH<'req>(self, handle_func: HandleFunc<'req>) -> Handler<'req> {
        Handler {
            route: self.into_handle_route(),
            GET: None,
            POST: None,
            PATCH: Some(handle_func),
            DELETE: None,
        }
    }
    fn DELETE<'req>(self, handle_func: HandleFunc<'req>) -> Handler<'req> {
        Handler {
            route: self.into_handle_route(),
            GET: None,
            POST: None,
            PATCH: None,
            DELETE: Some(handle_func),
        }
    }
}

impl Route for &'static str {
    fn into_handle_route(self) -> HandleRoute {
        if !self.starts_with('/') {
            tracing::error!("route `{self}` doesn't start with `/`");
            panic!()
        }

        let mut patterns = VecDeque::new();
        if self == "/" {
            return HandleRoute(patterns)
        }

        let mut pos = 1;
        for len in self[1..].split('/').map(|s| s.len()) {
            patterns.push_back(
                TriePattern::parse(pos..pos+len, self)
            );
            pos += len + 1;
        }
        HandleRoute(patterns)
    }
}
