use std::collections::VecDeque;
use crate::router::trie_tree::TriePattern;
use super::{Handlers, Handler};


pub trait Route: Sized {
    fn GET<'req>(self, handle_func: Handler<'req>) -> Handlers<'req>;
    fn POST<'req>(self, handle_func: Handler<'req>) -> Handlers<'req>;
    fn PATCH<'req>(self, handle_func: Handler<'req>) -> Handlers<'req>;
    fn DELETE<'req>(self, handle_func: Handler<'req>) -> Handlers<'req>;
    fn by<'req>(self, handlers: Handlers<'req>) -> Handlers<'req>;
} impl Route for &'static str {
    fn GET<'req>(self, handle_func: Handler<'req>) -> Handlers<'req> {
        Handlers {
            route: HandleRoute::parse(self),
            GET: Some(handle_func),
            POST: None,
            PATCH: None,
            DELETE: None,
        }
    }
    fn POST<'req>(self, handle_func: Handler<'req>) -> Handlers<'req> {
        Handlers {
            route: HandleRoute::parse(self),
            GET: None,
            POST: Some(handle_func),
            PATCH: None,
            DELETE: None,
        }
    }
    fn PATCH<'req>(self, handle_func: Handler<'req>) -> Handlers<'req> {
        Handlers {
            route: HandleRoute::parse(self),
            GET: None,
            POST: None,
            PATCH: Some(handle_func),
            DELETE: None,
        }
    }
    fn DELETE<'req>(self, handle_func: Handler<'req>) -> Handlers<'req> {
        Handlers {
            route: HandleRoute::parse(self),
            GET: None,
            POST: None,
            PATCH: None,
            DELETE: Some(handle_func),
        }
    }
    fn by<'req>(self, child: Handlers<'req>) -> Handlers<'req> {
        Handlers {
            route: HandleRoute::parse(self).merge(child.route),
            GET: child.GET,
            POST: child.POST,
            PATCH: child.PATCH,
            DELETE: child.DELETE,
        }
    }
}
pub struct HandleRoute(
    VecDeque<TriePattern>
); impl HandleRoute {
    fn parse(route_str: &'static str) -> Self {
        if !route_str.starts_with('/') {
            tracing::error!("route `{route_str}` doesn't start with `/`");
            panic!()
        }

        let mut patterns = VecDeque::new();
        if route_str == "/" {
            return HandleRoute(patterns)
        }

        let mut pos = 1;
        for len in route_str[1..].split('/').map(|s| s.len()) {
            patterns.push_back(
                TriePattern::parse(pos..pos+len, route_str)
            );
            pos += len + 1;
        }
        HandleRoute(patterns)
    }
    fn merge(self, child: Self) -> Self {
        for p in child {
            self.0.push_back(p)
        }
        self
    }
} const _: (/* HandleRoute impls */) = {
    impl Iterator for HandleRoute {
        type Item = TriePattern;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }
};
