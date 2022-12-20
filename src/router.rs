use crate::{
    components::method::Method, utils::map::StrMap, result::Result,
};
pub(self) use crate::{
    // === actual Handler ===
    server::Handler,
    // ======================
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
pub(crate) struct Router<'p> {
    GET:    Node<'p>,
    POST:   Node<'p>,
    PATCH:  Node<'p>,
    DELETE: Node<'p>,
}
impl<'p> Router<'p> {
    pub fn new() -> Self {
        Self {
            GET:    Node::new(Pattern::Str("")),
            POST:   Node::new(Pattern::Str("")),
            PATCH:  Node::new(Pattern::Str("")),
            DELETE: Node::new(Pattern::Str("")),
        }
    }
    pub fn register(&mut self,
        method:       Method,
        path_pattern: &'static str,
        handler:      Handler,
    ) -> std::result::Result<(), String> {
        let err_msg = format!("path pattern `{path_pattern}` is resistred duplicatedly");

        let mut path = path_pattern.split('/');
        { path.next(); }

        let tree = match method {
            Method::GET    => &mut self.GET,
            Method::POST   => &mut self.POST,
            Method::PATCH  => &mut self.PATCH,
            Method::DELETE => &mut self.DELETE,
        };
        
        tree.register(path, handler, err_msg)
    }
    pub fn search(&self,
        method:       Method,
        request_path: &'p str,
    ) -> Result<(&Handler, StrMap)> {
        let mut path = request_path.split('/');
        { path.next(); }

        let tree = match method {
            Method::GET    => &self.GET,
            Method::POST   => &self.POST,
            Method::PATCH  => &self.PATCH,
            Method::DELETE => &self.DELETE,
        };

        tree.search(path, StrMap::new())
    }
}
