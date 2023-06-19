use crate::{
    layer3_fang_handler::{Handler, Handlers, ByAnother, RouteSections, RouteSection, Fang},
};

type Range = std::ops::Range<usize>;
const _: () = {
    impl Into<Pattern> for RouteSection {
        fn into(self) -> Pattern {
            match self {
                RouteSection::Param => Pattern::Param,
                RouteSection::Static{ route, range } => Pattern::Static { route, range }
            }
        }
    }
};


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
    /// Why Option: root node doesn't have pattern
    pattern:  Option<Pattern>,
    fangs:    Vec<Fang>,
    handler:  Option<Handler>,
    children: Vec<Node>,
}

#[derive(Clone)]
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
            self.GET.register_handler(route.clone().into_iter(), handler)
        }
        if let Some(handler) = PUT {
            self.PUT.register_handler(route.clone().into_iter(), handler)
        }
        if let Some(handler) = POST {
            self.POST.register_handler(route.clone().into_iter(), handler)
        }
        if let Some(handler) = HEAD {
            self.HEAD.register_handler(route.clone().into_iter(), handler)
        }
        if let Some(handler) = PATCH {
            self.PATCH.register_handler(route.clone().into_iter(), handler)
        }
        if let Some(handler) = DELETE {
            self.DELETE.register_handler(route.clone().into_iter(), handler)
        }
        if let Some(handler) = OPTIONS {
            self.OPTIONS.register_handler(route.clone().into_iter(), handler)
        }

        self
    }

    pub(crate) fn merge_another(mut self, another: ByAnother) -> Self {
        let ByAnother { route, ohkami } = another;
        let another_routes = ohkami.routes;

        self.GET.merge_node(route.clone().into_iter(), another_routes.GET);
        self.PUT.merge_node(route.clone().into_iter(), another_routes.PUT);
        self.POST.merge_node(route.clone().into_iter(), another_routes.POST);
        self.HEAD.merge_node(route.clone().into_iter(), another_routes.HEAD);
        self.PATCH.merge_node(route.clone().into_iter(), another_routes.PATCH);
        self.DELETE.merge_node(route.clone().into_iter(), another_routes.DELETE);
        self.OPTIONS.merge_node(route.clone().into_iter(), another_routes.OPTIONS);
        
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
    fn register_handler(&mut self, mut route: <RouteSections as IntoIterator>::IntoIter, handler: Handler) {
        match route.next() {
            None => {self.handler.replace(handler);},
            Some(pattern) => match self.machable_child_mut(pattern.clone().into()) {
                Some(child) => child.register_handler(route, handler),
                None => {
                    let mut child = Node::new(pattern.into());
                    child.register_handler(route, handler);
                    self.children.push(child)
                }
            }
        }
    }

    fn merge_node(&mut self, mut route_to_merge_root: <RouteSections as IntoIterator>::IntoIter, another: Node) {
        compile_error!("USE `merge_here`");

        match route_to_merge_root.next() {
            None => self.children.push(another),
            Some(pattern) => match self.machable_child_mut(pattern.clone().into()) {
                Some(child) => child.merge_node(route_to_merge_root, another),
                None => {
                    let mut new_child = Node::new(pattern.into());
                    new_child.merge_node(route_to_merge_root, another);
                    self.children.push(new_child)
                }
            }
        }
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
    fn is_root(&self) -> bool {
        self.pattern.is_none()
    }

    fn machable_child_mut(&mut self, pattern: Pattern) -> Option<&mut Node> {
        for child in &mut self.children {
            if child.pattern.as_ref().unwrap().matches(&pattern) {
                return Some(child)
            }
        }
        None
    }

    fn merge_here(&mut self, another: Node) -> Result<(), String> {
        compile_error!("\
            Make `Fang`s have `id` that indicates identity of `Fang` function:\n\
            using\n\
            ```\n\
            fn id<F: Fn() -> Fut + 'static, Fut: Future<Output = ()>>(f: F) -> TypeId {\n\
            \0    <F as Any>::type_id(&f)\n\
            }\n\
            ```\n\
        ");
        /*
    pattern:  Option<Pattern>,
    fangs:    Vec<Fang>,
    handler:  Option<Handler>,
    children: Vec<Node>,
        */
        let pattern = match (&self.pattern, &another.pattern) {
            (None, None)                      => None,
            (Some(p), None) | (None, Some(p)) => Some(p.clone()),
            (Some(p1), Some(p2))              => p1.matches(p2).then_some(p1.clone()),
        };
        let fangs = ;

        Ok(())
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
