use crate::{
    components::method::Method,
    utils::range::RangeList,
    result::Result,
    handler::HandleFunc,
    setting::{Middleware, AfterMiddleware, BeforeMiddleware},
};

// === mock for test ===
// pub(self) type Handler = usize;
// =====================

mod pattern;
mod node; use node::Node;

use self::pattern::Pattern;

mod test_resister;
mod test_search;


// #[derive(PartialEq, Debug)]
#[allow(non_snake_case)]
pub(crate) struct Router {
    GET:    (Node, Vec<BeforeMiddleware>, Vec<AfterMiddleware>),
    POST:   (Node, Vec<BeforeMiddleware>, Vec<AfterMiddleware>),
    PATCH:  (Node, Vec<BeforeMiddleware>, Vec<AfterMiddleware>),
    DELETE: (Node, Vec<BeforeMiddleware>, Vec<AfterMiddleware>),
}
impl Router {
    pub(crate) fn new() -> Self {
        Self {
            GET:    (Node::new(Pattern::Nil), Vec::new(), Vec::new()),
            POST:   (Node::new(Pattern::Nil), Vec::new(), Vec::new()),
            PATCH:  (Node::new(Pattern::Nil), Vec::new(), Vec::new()),
            DELETE: (Node::new(Pattern::Nil), Vec::new(), Vec::new()),
        }
    }

    pub(crate) fn register(&mut self,
        method:       Method,
        path_pattern: &'static str /* already validated */,
        handler:      HandleFunc,
    ) -> std::result::Result<(), String> {
        let err_msg = format!(
            "path pattern `{}` is resistred duplicatedly",
            if path_pattern == "" {"/"} else {path_pattern}
        );

        let mut path = path_pattern.split('/');
        { path.next(); }

        let tree = match method {
            Method::GET    => &mut self.GET.0,
            Method::POST   => &mut self.POST.0,
            Method::PATCH  => &mut self.PATCH.0,
            Method::DELETE => &mut self.DELETE.0,
        };
        
        tree.register_handler(path, handler, err_msg)
    }
    pub(crate) fn search<'req>(&self,
        method:       Method,
        request_path: &'req str,
    ) -> Result<(
        &HandleFunc,
        RangeList,
        Vec<&BeforeMiddleware>,
        Vec<&AfterMiddleware>,
    )> {
        let mut path = request_path.split('/');
        { path.next(); }

        let offset = method.len();

        let (tree, init_before, init_after) = match method {
            Method::GET    => &self.GET,
            Method::POST   => &self.POST,
            Method::PATCH  => &self.PATCH,
            Method::DELETE => &self.DELETE,
        };

        let mut before_middleware = Vec::with_capacity(init_before.len());
        for f in init_before {before_middleware.push(f)}

        let mut after_middleware = Vec::with_capacity(init_after.len());
        for f in init_after {after_middleware.push(f)}

        tree.search(path, RangeList::new(), offset, before_middleware, after_middleware)
    }

    pub(crate) fn apply(mut self, middlware: Middleware) -> std::result::Result<Self, String> {
        if ! middlware.setup_errors.is_empty() {
            return Err(
                middlware.setup_errors
                    .into_iter()
                    .fold(String::new(), |it, next| it + &next + "\n")
            )
        }

        for (method, route, mut store) in middlware.before {
            if route == "*" {
                match method {
                    Method::GET    => self.GET.1.push(store.pop().unwrap()),
                    Method::POST   => self.POST.1.push(store.pop().unwrap()),
                    Method::PATCH  => self.PATCH.1.push(store.pop().unwrap()),
                    Method::DELETE => self.DELETE.1.push(store.pop().unwrap()),
                }
                drop(store)
            } else {
                let err_msg = format!(
                    "Failed to resister before-handling middleware func for route `{route}`. If you got this error, please report to https://github.com/kana-rus/ohkami/issues"
                );
                match method {
                    Method::GET    => self.GET.0 = self.GET.0.register_before_middleware(route, store, err_msg)?,
                    Method::POST   => self.POST.0 = self.POST.0.register_before_middleware(route, store, err_msg)?,
                    Method::PATCH  => self.PATCH.0 = self.PATCH.0.register_before_middleware(route, store, err_msg)?,
                    Method::DELETE => self.DELETE.0 = self.DELETE.0.register_before_middleware(route, store, err_msg)?,
                }
            }
        }

        for (method, route, mut store) in middlware.after {
            if route == "*" {
                match method {
                    Method::GET    => self.GET.2.push(store.pop().unwrap()),
                    Method::POST   => self.POST.2.push(store.pop().unwrap()),
                    Method::PATCH  => self.PATCH.2.push(store.pop().unwrap()),
                    Method::DELETE => self.DELETE.2.push(store.pop().unwrap()),
                }
                drop(store)
            } else {
                let err_msg = format!(
                    "Failed to resister after-handling middleware func for route `{route}`. If you got this error, please report to https://github.com/kana-rus/ohkami/issues"
                );
                match method {
                    Method::GET    => self.GET.0 = self.GET.0.register_after_middleware(route, store, err_msg)?,
                    Method::POST   => self.POST.0 = self.POST.0.register_after_middleware(route, store, err_msg)?,
                    Method::PATCH  => self.PATCH.0 = self.PATCH.0.register_after_middleware(route, store, err_msg)?,
                    Method::DELETE => self.DELETE.0 = self.DELETE.0.register_after_middleware(route, store, err_msg)?,
                }
            }
        }

        Ok(self)
    }
}
