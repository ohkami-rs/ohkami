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
    GET:    Node,
    POST:   Node,
    PATCH:  Node,
    DELETE: Node,
}
impl Router {
    pub(crate) fn new() -> Self {
        Self {
            GET:    Node::new(Pattern::Nil),
            POST:   Node::new(Pattern::Nil),
            PATCH:  Node::new(Pattern::Nil),
            DELETE: Node::new(Pattern::Nil),
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
            Method::GET    => &mut self.GET,
            Method::POST   => &mut self.POST,
            Method::PATCH  => &mut self.PATCH,
            Method::DELETE => &mut self.DELETE,
        };
        
        tree.register_handler(path, handler, err_msg)
    }
    pub(crate) fn search<'req>(&self,
        method:       Method,
        request_path: &'req str,
    ) -> Result<(
        &HandleFunc,
        RangeList,
        &Vec<BeforeMiddleware>,
        &Vec<AfterMiddleware>,
    )> {
        let mut path = request_path.split('/');
        { path.next(); }

        let offset = method.len();

        match method {
            Method::GET    => &self.GET,
            Method::POST   => &self.POST,
            Method::PATCH  => &self.PATCH,
            Method::DELETE => &self.DELETE,
        }.search(
            path, RangeList::new(), offset
        )
    }

    pub(crate) fn apply(mut self, middlware: Middleware) -> std::result::Result<Self, String> {
        if ! middlware.setup_errors.is_empty() {
            return Err(
                middlware.setup_errors
                    .into_iter()
                    .fold(String::new(), |it, next| it + &next + "\n")
            )
        }

        for (method, route, store, is_from_any) in middlware.before {
            let err_msg = format!("Failed to resister a before-handling middleware function for `{} {route}`. Please report this: https://github.com/kana-rus/ohkami/issues",
                if is_from_any {"{any method}".to_owned()} else {method.to_string()});
            // let warn_msg = format!("A before-handling middleware function for `{} {route}` won't work to any reuqest. No handlerthat matches this is resistered.",
            //     if is_from_any {"{any method}".to_owned()} else {method.to_string()});
            match method {
                Method::GET    => self.GET = self.GET.register_before_middleware(route, store, err_msg, /*warn_msg*/)?,
                Method::POST   => self.POST = self.POST.register_before_middleware(route, store, err_msg, /*warn_msg*/)?,
                Method::PATCH  => self.PATCH = self.PATCH.register_before_middleware(route, store, err_msg, /*warn_msg*/)?,
                Method::DELETE => self.DELETE = self.DELETE.register_before_middleware(route, store, err_msg, /*warn_msg*/)?,
            }
        }

        for (method, route, store, is_from_any) in middlware.after {
            let err_msg = format!("Failed to resister a before-handling middleware function for `{} {route}`. Please report this: https://github.com/kana-rus/ohkami/issues",
                if is_from_any {"{any method}".to_owned()} else {method.to_string()});
            // let warn_msg = format!("A before-handling middleware function for `{} {route}` won't work to any reuqest. No handlerthat matches this is resistered.",
            //     if is_from_any {"{any method}".to_owned()} else {method.to_string()});
            match method {
                Method::GET    => self.GET = self.GET.register_after_middleware(route, store, err_msg, /*warn_msg*/)?,
                Method::POST   => self.POST = self.POST.register_after_middleware(route, store, err_msg, /*warn_msg*/)?,
                Method::PATCH  => self.PATCH = self.PATCH.register_after_middleware(route, store, err_msg, /*warn_msg*/)?,
                Method::DELETE => self.DELETE = self.DELETE.register_after_middleware(route, store, err_msg, /*warn_msg*/)?,
            }
        }

        Ok(self)
    }
}
