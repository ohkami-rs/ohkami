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
    pub(super/* for test */) GET:     Node,
    pub(super/* for test */) PUT:     Node,
    pub(super/* for test */) POST:    Node,
    pub(super/* for test */) HEAD:    Node,
    pub(super/* for test */) PATCH:   Node,
    pub(super/* for test */) DELETE:  Node,
    pub(super/* for test */) OPTIONS: Node,
}

pub(super/* for test */) struct Node {
    /// Why Option: root node doesn't have pattern
    pub(super/* for test */) pattern:  Option<Pattern>,
    pub(super/* for test */) fangs:    Vec<Fang>,
    pub(super/* for test */) handler:  Option<Handler>,
    pub(super/* for test */) children: Vec<Node>,
}

#[derive(Clone)]
pub(super/* for test */) enum Pattern {
    Static{ route: &'static [u8], range: Range },
    Param,
} const _: () = {
    impl std::fmt::Debug for Pattern {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Param                   => f.write_str(":Param"),
                Self::Static { route, range } => f.write_str(&format!(
                    "'{}'", std::str::from_utf8(&route[range.clone()]).unwrap()
                )),
            }
        }
    }
};


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
        let another_routes = {
            let mut routes = ohkami.routes;
            for fang in ohkami.fangs {
                routes = routes.apply_fang(fang)
            }
            routes
        };

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
        super::RadixRouter {
            GET:     self.GET.into_radix(),
            PUT:     self.PUT.into_radix(),
            POST:    self.POST.into_radix(),
            HEAD:    self.HEAD.into_radix(),
            PATCH:   self.PATCH.into_radix(),
            DELETE:  self.DELETE.into_radix(),
            OPTIONS: self.OPTIONS.into_radix(),
        }
    }
}

impl Node {
    fn register_handler(&mut self, mut route: <RouteSections as IntoIterator>::IntoIter, handler: Handler) {
        #[cfg(debug_assertions)] println!("[register_handler] route: {route:?}");

        match route.next() {
            None => if self.handler.replace(handler).is_some() {panic!("Conflicting handler registeration")},
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
        match route_to_merge_root.next() {
            None => if let Err(e) = self.merge_here(another) {panic!("Can't merge nodes: {e}")},
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
        for child in &mut self.children {
            child.apply_fang(fang.clone())
        }
        if self.handler.is_some() {
            self.fangs.push(fang)
        }
    }

    fn into_radix(self) -> super::radix::Node {
        let Node { pattern, mut fangs, mut handler, mut children } = self;

        let mut patterns = match pattern {
            None          => vec![],
            Some(pattern) => vec![pattern],
        };

        if children.len() == 1
        && (handler.is_none() && children[0].handler.is_some()) {
            let Node {
                pattern:  child_pattern,
                fangs:    child_fangs,
                handler:  child_handler,
                children: child_children,
            } = children.pop(/* single child */).unwrap(/* `children` is empty here */);

            children = child_children;
            handler  = child_handler;
            
            let child_pattern = child_pattern.unwrap(/* `child` is not root */);
            if patterns.last().is_some_and(|last| last.is_static()) && child_pattern.is_static() {
                let (_, this_range) = patterns.pop(/*=== POPing here ===*/).unwrap().to_static().unwrap();
                let (route, child_range) = child_pattern.to_static().unwrap();

                patterns.push(Pattern::Static {
                    route,
                    range: (this_range.start..child_range.end),
                })
            } else {
                patterns.push(child_pattern)
            }

            #[cfg(debug_assertions)]
            println!("[into_radix] merged patterns: {patterns:?}");

            for cf in child_fangs {
                if fangs.iter().all(|f| f.id() != cf.id()) {
                    fangs.push(cf)
                }
            }
        }

        super::radix::Node {
            handler,
            children: children.into_iter().map(|c| c.into_radix()).collect(),
            front: Box::leak(fangs
                .into_iter().map(|f| match f {
                    Fang::Front(f) => f,
                }).collect()
            ),
            patterns: Box::leak(patterns
                .into_iter()
                .map(Pattern::into_radix)
                .collect()
            ),
        }
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

    fn machable_child_mut(&mut self, pattern: Pattern) -> Option<&mut Node> {
        for child in &mut self.children {
            if child.pattern.as_ref().unwrap().matches(&pattern) {
                return Some(child)
            }
        }
        None
    }

    fn merge_here(&mut self, another: Node) -> Result<(), String> {
        let Node {
            pattern:  another_pattern,
            fangs:    another_fangs,
            handler:  another_handler,
            children: another_children,
        } = another;

        self.pattern = match (&self.pattern, &another_pattern) {
            (None, None)                      => None,
            (Some(p), None) | (None, Some(p)) => Some(p.clone()),
            (Some(p1), Some(p2))              => p1.matches(p2).then_some(p1.clone()),
        };

        for af in another_fangs {
            if self.fangs.iter().all(|f| f.id() != af.id()) {
                self.fangs.push(af)
            }
        }

        if self.handler.is_none() {
            if let Some(ah) = another_handler {
                self.handler.replace(ah);
            }
        } else {
            if another_handler.is_some() {
                return Err(format!("Conflicting handler registeration"));
            }
        }

        for ac in another_children {
            self.children.push(ac)
        }

        Ok(())
    }
}

impl Pattern {
    fn is_param(&self) -> bool {
        match self {
            Self::Param => true,
            Self::Static{..} => false,
        }
    }
    fn is_static(&self) -> bool {
        match self {
            Self::Static{..} => true,
            Self::Param => false,
        }
    }

    fn to_static(self) -> Option<(&'static [u8], Range)> {
        match self {
            Self::Param => None,
            Self::Static{ route, range } => Some((route, range))
        }
    }

    fn read_as_static(&self) -> Option<&[u8]> {
        match self {
            Self::Param => None,
            Self::Static{ route, range } => Some(&route[(range.start)..(range.end)])
        }
    }

    fn matches(&self, another: &Self) -> bool {
        match self {
            Self::Param => another.is_param(),
            Self::Static{..} => self.read_as_static() == another.read_as_static(),
        }
    }

    fn into_radix(self) -> super::radix::Pattern {
        match self {
            Self::Param                  => super::radix::Pattern::Param,
            Self::Static{ route, range } => super::radix::Pattern::Static(&route[range]),
        }
    }
}
