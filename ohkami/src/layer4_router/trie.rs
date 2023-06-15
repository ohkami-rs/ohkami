use crate::layer3_fang_handler::{Handler, Handlers, ByAnother, RouteSections, Fang};
type Range = std::ops::Range<usize>;


/*===== defs =====*/
pub struct TrieRouter {
    GET: Node,
    PUT: Node,
    POST: Node,
    HEAD: Node,
    PATCH: Node,
    DELETE: Node,
    OPTIONS: Node,
}

struct Node {
    /// why Option: root nodes don't have pattern
    pattern:  Option<Pattern>,
    fangs:    Vec<Fang>,
    handler:  Option<Handler>,
    children: Vec<Node>,
}

enum Pattern {
    Static{ route: &'static [u8], range: Range },
    Param,
}


/*===== impls =====*/
impl TrieRouter {
    pub(crate) fn new() -> Self {
        Self {
            GET:     Node::root(),
            PUT:     Node::root(),
            POST:    Node::root(),
            HEAD:    Node::root(),
            PATCH:   Node::root(),
            DELETE:  Node::root(),
            OPTIONS: Node::root(),
        }
    }

    pub(crate) fn register_handlers(mut self, handlers: Handlers) -> Self {
        let Handlers { route, GET, PUT, POST, HEAD, PATCH, DELETE, OPTIONS } = handlers;

        if let Some(handler) = GET {
            self.GET.register_handler(route.clone(), handler)
        }
        if let Some(handler) = PUT {
            self.PUT.register_handler(route.clone(), handler)
        }
        if let Some(handler) = POST {
            self.POST.register_handler(route.clone(), handler)
        }
        if let Some(handler) = HEAD {
            self.HEAD.register_handler(route.clone(), handler)
        }
        if let Some(handler) = PATCH {
            self.PATCH.register_handler(route.clone(), handler)
        }
        if let Some(handler) = DELETE {
            self.DELETE.register_handler(route.clone(), handler)
        }
        if let Some(handler) = OPTIONS {
            self.OPTIONS.register_handler(route.clone(), handler)
        }

        self
    }

    pub(crate) fn merge_another(mut self, another: ByAnother) -> Self {
        let ByAnother { route, ohkami } = another;
        let another_routes = ohkami.routes;

        self.GET.merge_node(another_routes.GET);
        self.PUT.merge_node(another_routes.PUT);
        self.POST.merge_node(another_routes.POST);
        self.HEAD.merge_node(another_routes.HEAD);
        self.PATCH.merge_node(another_routes.PATCH);
        self.DELETE.merge_node(another_routes.DELETE);
        self.OPTIONS.merge_node(another_routes.OPTIONS);
        
        self
    }

    pub(crate) fn apply_fang(mut self, fang: Fang) -> Self {
        self.GET.apply_fang(fang.clone());
        self.PUT.apply_fang(fang.clone());
        self.POST.apply_fang(fang.clone());
        self.HEAD.apply_fang(fang.clone());
        self.PATCH.apply_fang(fang.clone());
        self.DELETE.apply_fang(fang.clone());
        self.OPTIONS.apply_fang(fang.clone());

        self
    }

    pub(crate) fn into_radix(self) -> super::RadixRouter {
        todo!()
    }
}

impl Node {
    fn register_handler(&mut self, route: RouteSections, handler: Handler) {
        // compile_error!(TODO)
        todo!()
    }

    fn merge_node(&mut self, another: Node) {
        // compile_error!(TODO)
        todo!()
    }

    fn apply_fang(&mut self, fang: Fang) {
        // compile_error!(TODO)
        todo!()
    }
}


/*===== utils =====*/
impl Node {
    fn new(pattern: Pattern) -> Self {
        Self {
            pattern:  Some(pattern),
            handler:  None,
            fangs:    vec![],
            children: vec![],
        }
    }
    fn root() -> Self {
        Self {
            pattern:  None,
            handler:  None,
            fangs:    vec![],
            children: vec![],
        }
    }

    fn machable_child_mut(&mut self, pattern: &Pattern) -> Option<&mut Node> {
        for child in &mut self.children {
            if child.pattern.as_ref().unwrap().matches(pattern) {
                return Some(child)
            }
        }
        None
    }
}

impl Pattern {
    #[inline(always)] fn is_static(&self) -> bool {
        match self {
            Self::Static{..} => true,
            _ => false,
        }
    }
    #[inline(always)] fn is_param(&self) -> bool {
        match self {
            Self::Param => true,
            Self::Static{..} => false,
        }
    }
    #[inline(always)] fn as_static(&self) -> Option<&[u8]> {
        match self {
            Self::Param => None,
            Self::Static{ route, range } => Some(&route[(range.start)..(range.end)])
        }
    }
    #[inline(always)] fn matches(&self, another: &Self) -> bool {
        match self {
            Self::Param => another.is_param(),
            Self::Static{..} => self.as_static() == another.as_static(),
        }
    }
}
