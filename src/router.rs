use std::str::Split;

use crate::{
    server::Handler,
    components::method::Method,
};

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
            GET:    Node::new(""),
            POST:   Node::new(""),
            PATCH:  Node::new(""),
            DELETE: Node::new(""),
        }
    }
    pub fn register(&mut self,
        method:       Method,
        path_pattern: &'static str,
        handler:      Handler,
    ) -> std::result::Result<(), String> {
        match method {
            Method::GET    => &mut self.GET,
            Method::POST   => &mut self.POST,
            Method::PATCH  => &mut self.PATCH,
            Method::DELETE => &mut self.DELETE,
        }.register(
            path_pattern, handler
        )
    }
}

enum Pattern<'p> {
    Any,
    Str(&'p str),
    Param(&'p str),
} impl<'p> Pattern<'p> {
    fn from(section: &'p str) -> Self {
        match section {
            "*" => Self::Any,
            p if p.starts_with(':') => Self::Param(&p[1..]),
            p => Self::Str(p),
        }
    }

    fn matches(&self, section: &str) -> bool {
        let pattern = Pattern::from(section);
        match self {
            Pattern::Any => true,
            Pattern::Param(_) => pattern.is_param(),
            Pattern::Str(p) => p == &section,
        }
    }
    fn is_param(&self) -> bool {
        match self {
            Pattern::Param(_) => true,
            _ => false,
        }
    }
    fn is_str(&self) -> bool {
        match self {
            Pattern::Str(_) => true,
            _ => false,
        }
    }
}

struct Node<'p> {
    pattern:  Pattern<'p>,
    handler:  Option<Handler>,
    children: Vec<Node<'p>>,
} impl<'p> Node<'p> {
    fn new(section: &'p str) -> Self {
        Self {
            pattern:  Pattern::from(section),
            handler:  None,
            children: Vec::new(),
        }
    }

    fn register(&mut self,
        path:    &'p str, // already validated
        handler: Handler,
    ) -> std::result::Result<(), String> {
        let err_msg = format!("path pattern `{path}` is resistred duplicatedly");

        let mut path = path.split('/');
        { path.next(); /* all valid paths start with '/' , so if they are split by '/',
        the result ( Split<'_, char> ) starts with Some(""). Here discard this */ }

        self._register(path, handler, err_msg)
    }
    fn _register(&mut self,
        mut path: Split<'p, char>,
        handler:  Handler,
        err_msg:  String,
    ) -> std::result::Result<(), String> {
        if let Some(section) = path.next() {
            if let Some(child) = 'search: {
                for child in &mut self.children {
                    if child.pattern.matches(section) {
                        break 'search Some(child)
                    }
                }
                None
            } {
                child._register(path, handler, err_msg)

            } else {
                let mut new_branch = Node::new(section);
                new_branch.attach(path, handler);
                self.children.push(new_branch);
                Ok(())
            }

        } else {
            Err(err_msg)
        }
    }

    fn attach(&mut self,
        path:    Split<'p, char>,
        handler: Handler,
    ) {
        let path = path.rev().collect::<Vec<_>>();
        self._attach(path, handler)
    }
    fn _attach(&mut self,
        mut path: Vec<&'p str>,
        handler:  Handler,
    ) {
        if let Some(section) = path.pop() {
            let mut new_node = Node::new(section);
            new_node._attach(path, handler);
            self.children.push(new_node)
        } else {
            self.handler = Some(handler)
        }
    }
}
