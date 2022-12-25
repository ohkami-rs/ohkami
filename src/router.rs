use crate::{
    components::method::Method, utils::range::RangeList, result::Result, handler::HandleFunc, setting::{Middleware, MiddlewareFunc},
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
    GET:    (Vec<MiddlewareFunc>, Node),
    POST:   (Vec<MiddlewareFunc>, Node),
    PATCH:  (Vec<MiddlewareFunc>, Node),
    DELETE: (Vec<MiddlewareFunc>, Node),
}
impl Router {
    pub(crate) fn new() -> Self {
        Self {
            GET:    (Vec::new(), Node::new(Pattern::Nil)),
            POST:   (Vec::new(), Node::new(Pattern::Nil)),
            PATCH:  (Vec::new(), Node::new(Pattern::Nil)),
            DELETE: (Vec::new(), Node::new(Pattern::Nil)),
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
            Method::GET    => &mut self.GET.1,
            Method::POST   => &mut self.POST.1,
            Method::PATCH  => &mut self.PATCH.1,
            Method::DELETE => &mut self.DELETE.1,
        };
        
        tree.register_handler(path, handler, err_msg)
    }
    pub(crate) fn search<'req>(&self,
        method:       Method,
        request_path: &'req str,
    ) -> Result<(
        &HandleFunc,
        RangeList,
        Vec<&MiddlewareFunc>,
        Option<&MiddlewareFunc>,
    )> {
        let mut path = request_path.split('/');
        { path.next(); }

        let offset = method.len();

        let (init_proc, tree) = match method {
            Method::GET    => &self.GET,
            Method::POST   => &self.POST,
            Method::PATCH  => &self.PATCH,
            Method::DELETE => &self.DELETE,
        };

        let mut middleware_proccess = Vec::new();
        for proc in init_proc {
            middleware_proccess.push(proc)
        }

        tree.search(path, RangeList::new(), offset, middleware_proccess)
    }

    pub(crate) fn apply(mut self, middlware: Middleware) -> std::result::Result<Self, String> {
        if ! middlware.setup_errors.is_empty() {
            return Err(
                middlware.setup_errors
                    .into_iter()
                    .fold(String::new(), |it, next| it + &next + "\n")
            )
        }

        for (method, route, func) in middlware.proccess {
            let error_msg = format!("middleware func just for `{method} {route}` is registered duplicatedly");

            if route == "*" {
                match method {
                    Method::GET    => self.GET.0.push(func),
                    Method::POST   => self.POST.0.push(func),
                    Method::PATCH  => self.PATCH.0.push(func),
                    Method::DELETE => self.DELETE.0.push(func),
                }
            } else {
                match method {
                    Method::GET    => self.GET.1 = self.GET.1.register_middleware_func(route, func, error_msg)?,
                    Method::POST   => self.POST.1 = self.POST.1.register_middleware_func(route, func, error_msg)?,
                    Method::PATCH  => self.PATCH.1 = self.PATCH.1.register_middleware_func(route, func, error_msg)?,
                    Method::DELETE => self.DELETE.1 = self.DELETE.1.register_middleware_func(route, func, error_msg)?,
                }
            }
        }

        Ok(self)
    }
}
