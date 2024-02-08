use std::borrow::Cow;
use super::{RouteSection, RouteSections};
use crate::{
    Method,
    fang::{proc::FangProc, Fang},
    handler::{ByAnother, Handler, Handlers}
};

const _: () = {
    impl Into<Pattern> for RouteSection {
        fn into(self) -> Pattern {
            match self {
                RouteSection::Param         => Pattern::Param,
                RouteSection::Static(bytes) => Pattern::Static(Cow::Borrowed(bytes))
            }
        }
    }
};


/*===== defs =====*/
#[derive(Clone/* for testing */)]
pub struct TrieRouter {
    pub(super/* for test */) GET:     Node,
    pub(super/* for test */) PUT:     Node,
    pub(super/* for test */) POST:    Node,
    pub(super/* for test */) PATCH:   Node,
    pub(super/* for test */) DELETE:  Node,
    pub(super) HEADfangs:    Vec<Fang>,
    pub(super) OPTIONSfangs: Vec<Fang>,
}

#[derive(Clone/* for testing */)]
pub(super/* for test */) struct Node {
    /// Why Option: root node doesn't have pattern
    pub(super/* for test */) pattern:  Option<Pattern>,
    pub(super/* for test */) fangs:    Vec<Fang>,
    pub(super/* for test */) handler:  Option<Handler>,
    pub(super/* for test */) children: Vec<Node>,
}

#[derive(Clone)]
pub(super/* for test */) enum Pattern {
    Static(Cow<'static, [u8]>),
    Param,
} const _: () = {
    impl std::fmt::Debug for Pattern {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Param     => f.write_str(":Param"),
                Self::Static(v) => f.write_str(&format!(
                    "'{}'", std::str::from_utf8(&v).unwrap()
                )),
            }
        }
    }

    impl PartialEq for Pattern {
        fn eq(&self, other: &Self) -> bool {
            match self {
                Self::Param => match other {
                    Self::Param => true,
                    _ => false,
                }
                Self::Static(this_bytes) => {
                    match other {
                        Self::Static(other_bytes) => this_bytes == other_bytes,
                        _ => false
                    }
                }
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
            PATCH:   Node::root(),
            DELETE:  Node::root(),
            HEADfangs:    Vec::new(),
            OPTIONSfangs: Vec::new(),
        }
    }

    pub(crate) fn register_handlers(mut self, handlers: Handlers) -> Self {
        let Handlers { route, GET, PUT, POST, PATCH, DELETE } = handlers;

        if let Some(handler) = GET {
            if let Err(e) = self.GET.register_handler(route.clone().into_iter(), handler) {panic!("{e}")}
        }
        if let Some(handler) = PUT {
            if let Err(e) = self.PUT.register_handler(route.clone().into_iter(), handler) {panic!("{e}")}
        }
        if let Some(handler) = POST {
            if let Err(e) = self.POST.register_handler(route.clone().into_iter(), handler) {panic!("{e}")}
        }
        if let Some(handler) = PATCH {
            if let Err(e) = self.PATCH.register_handler(route.clone().into_iter(), handler) {panic!("{e}")}
        }
        if let Some(handler) = DELETE {
            if let Err(e) = self.DELETE.register_handler(route.clone().into_iter(), handler) {panic!("{e}")}
        }

        self
    }

    pub(crate) fn merge_another(mut self, another: ByAnother) -> Self {
        let ByAnother { route, ohkami } = another;
        let another_routes = ohkami.into_router();

        self.GET   .merge_node(route.clone().into_iter(), another_routes.GET   ).unwrap();
        self.PUT   .merge_node(route.clone().into_iter(), another_routes.PUT   ).unwrap();
        self.POST  .merge_node(route.clone().into_iter(), another_routes.POST  ).unwrap();
        self.PATCH .merge_node(route.clone().into_iter(), another_routes.PATCH ).unwrap();
        self.DELETE.merge_node(route.clone().into_iter(), another_routes.DELETE).unwrap();

        for af in another_routes.HEADfangs {
            self.HEADfangs.push(af);
        }
        for af in another_routes.OPTIONSfangs {
            self.OPTIONSfangs.push(af);
        }
        
        self
    }

    pub(crate) fn apply_fang(mut self, methods: &'static [Method], fang: Fang) -> Self {
        for method in methods {
            match method {
                Method::GET     => self.GET         .apply_fang(fang.clone()),
                Method::PUT     => self.PUT         .apply_fang(fang.clone()),
                Method::POST    => self.POST        .apply_fang(fang.clone()),
                Method::PATCH   => self.PATCH       .apply_fang(fang.clone()),
                Method::DELETE  => self.DELETE      .apply_fang(fang.clone()),
                Method::HEAD    => self.HEADfangs   .push(fang.clone()),
                Method::OPTIONS => self.OPTIONSfangs.push(fang.clone()),
            }
        }

        self
    }

    pub(crate) fn into_radix(self) -> super::RadixRouter {
        super::RadixRouter {
            GET:    self.GET   .into_radix(),
            PUT:    self.PUT   .into_radix(),
            POST:   self.POST  .into_radix(),
            PATCH:  self.PATCH .into_radix(),
            DELETE: self.DELETE.into_radix(),
            HEADfangs: {
                let (mut front, mut back) = (vec![], vec![]);
                for f in self.HEADfangs {
                    match f.proc {
                        FangProc::Front(ff) => front.push(ff),
                        FangProc::Back(bf)  => back .push(bf),
                    }
                }
                (Box::leak(front.into_boxed_slice()), Box::leak(back.into_boxed_slice()))
            },
            OPTIONSfangs: {
                let (mut front, mut back) = (vec![], vec![]);
                for f in self.OPTIONSfangs {
                    match f.proc {
                        FangProc::Front(ff) => front.push(ff),
                        FangProc::Back(bf)  => back .push(bf),
                    }
                }
                (Box::leak(front.into_boxed_slice()), Box::leak(back.into_boxed_slice()))
            }
        }
    }
}

impl Node {
    fn register_handler(&mut self, mut route: <RouteSections as IntoIterator>::IntoIter, handler: Handler) -> Result<(), String> {
        match route.next() {
            None => {
                self.set_handler(handler)?;
                Ok(())
            }
            Some(pattern)   => match self.machable_child_mut(pattern.clone().into()) {
                Some(child) => child.register_handler(route, handler),
                None        => {
                    let mut child = Node::new(pattern.into());
                    child.register_handler(route, handler)?;
                    self.append_child(child)?;
                    Ok(())
                }
            }
        }
    }

    fn merge_node(&mut self, mut route_to_merge_root: <RouteSections as IntoIterator>::IntoIter, another: Node) -> Result<(), String> {
        match route_to_merge_root.next() {
            None => {
                self.merge_here(another)?;
                Ok(())
            }
            Some(pattern) => match self.machable_child_mut(pattern.clone().into()) {
                Some(child) => child.merge_node(route_to_merge_root, another),
                None => {
                    let mut new_child = Node::new(pattern.into());
                    new_child.merge_node(route_to_merge_root, another)?;
                    self.append_child(new_child)?;
                    Ok(())
                }
            }
        }
    }

    fn apply_fang(&mut self, fang: Fang) {
        fn apply_front_fang(this: &mut Node, fang: Fang) {
            assert!(fang.is_front());

            this.append_fang(fang)
        }
        fn apply_back_fang(this: &mut Node, fang: Fang) {
            assert!( ! fang.is_front());

            for child in &mut this.children {
                apply_back_fang(child, fang.clone())
            }
            this.append_fang(fang)
        }

        if fang.is_front() {
            apply_front_fang(self, fang)
        } else {
            apply_back_fang (self, fang)
        }
    }

    fn into_radix(self) -> super::radix::Node {
        let Node { pattern, mut fangs, mut handler, mut children } = self;

        let mut patterns = pattern.into_iter().collect::<Vec<_>>();

        while children.len() == 1 && handler.is_none() {
            let Node {
                pattern:  child_pattern,
                fangs:    child_fangs,
                handler:  child_handler,
                children: child_children,
            } = children.pop(/* pop the single child */).unwrap(/* `children` is empty here */);

            children = child_children;
            handler  = child_handler;
            for cf in child_fangs {
                fangs.push(cf);
            }
            
            let child_pattern = child_pattern.unwrap(/* `child` is not root */);
            if patterns.last().is_some_and(|last| last.is_static()) && child_pattern.is_static() {
                let last_pattern = patterns.pop(/*=== POPing here ===*/).unwrap();
                let this_static  = last_pattern.to_static().unwrap();
                let child_static = child_pattern.to_static().unwrap();

                patterns.push(Pattern::Static(
                    Cow::Owned([this_static, b"/", child_static].concat())
                ));
            } else {
                patterns.push(child_pattern)
            }
        }

        let (mut front, mut back) = (Vec::new(), Vec::new());
        {
            let mut unique_fangs = Vec::new();
            for f in fangs {
                if unique_fangs.iter().all(|uf| uf != &f) {
                    unique_fangs.push(f)
                }
            }

            for uf in unique_fangs {
                match uf.proc {
                    FangProc::Front(ff) => front.push(ff),
                    FangProc::Back (bf) => back.push(bf),
                }
            }
        }

        super::radix::Node {
            handler,
            children: children.into_iter().map(|c| c.into_radix()).collect(),
            front:    Box::leak(front.into_boxed_slice()),
            back:     Box::leak(back .into_boxed_slice()),
            patterns: Box::leak(patterns
                .into_iter()
                .map(Pattern::into_radix)
                .collect())
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

    /// Called in following situation :
    /// 
    /// <br/>
    /// 
    /// ```ignore
    /// TrieRouter::new()
    ///     .register_handlers("/hc".GET(health_check))
    ///     .register_handlers("/api".by(
    ///         Ohkami::new()(
    ///             "/"         .GET (hello),
    ///             "/users"    .POST(create_user),
    ///             "/users/:id".GET (get_user),
    ///             "/tasks"    .GET (get_task),
    ///         )
    ///     ))
    /// ```
    /// 
    /// <br/>
    /// 
    /// This must equals :
    /// 
    /// <br/>
    /// 
    /// ```ignore
    /// TrieRouter::new()
    ///     .register_handlers("/hc"           .GET (health_check))
    ///     .register_handlers("/api"          .GET (hello))
    ///     .register_handlers("/api/users"    .POST(create_user))
    ///     .register_handlers("/api/users/:id".GET (get_user))
    ///     .register_handlers("/api/tasks/:id".GET (get_task));
    /// ```
    fn merge_here(&mut self, another_root: Node) -> Result<(), String> {
        let Node {
            pattern:  None, // <-- another_root は root node のはずなので必ず None のはず
            fangs:    another_root_fangs,
            handler:  another_root_handler,
            children: another_children,
        } = another_root else {
            panic!("Unexpectedly called `Node::merge_here` where `another_root` is not root node")
        };

        if let Some(new_handler) = another_root_handler {
            self.set_handler(new_handler)?
        }

        for arf in another_root_fangs {
            self.append_fang(arf)
        }

        for ac in another_children {
            self.append_child(ac)?
        }

        Ok(())
    }
}

impl Node {
    fn append_child(&mut self, new_child: Node) -> Result<(), String> {
        match new_child.pattern.as_ref().expect("Invalid child node: Child node must have pattern") {
            Pattern::Param => {
                self.children.push(new_child);
                Ok(())
            }
            Pattern::Static(bytes) => {
                if self.children.iter().find(|c| c.pattern.as_ref().unwrap().to_static().is_some_and(|p| p == bytes.as_ref())).is_some() {
                    let __position__ = match &self.pattern {
                        None    => format!("For the first part of route"),
                        Some(p) => format!("After {p:?}"),
                    };
                    Err(format!("Conflicting route definition: {__position__}, pattern '{}' is registered twice", std::str::from_utf8(&bytes).unwrap()))
                } else {
                    self.children.push(new_child);
                    Ok(())
                }
            }
        }
    }

    fn append_fang(&mut self, new_fang: Fang) {
        self.fangs.push(new_fang);
    }

    fn set_handler(&mut self, new_handler: Handler) -> Result<(), String> {
        if self.handler.is_some() {
            return Err(format!("Conflicting handler registering"))
        }
        self.handler = Some(new_handler);
        Ok(())
    }
}

impl Pattern {
    fn is_param(&self) -> bool {
        match self {
            Self::Param => true,
            Self::Static(_) => false,
        }
    }
    fn is_static(&self) -> bool {
        match self {
            Self::Static(_) => true,
            Self::Param => false,
        }
    }

    fn to_static(&self) -> Option<&[u8]> {
        match self {
            Self::Param         => None,
            Self::Static(bytes) => Some(&bytes)
        }
    }

    fn matches(&self, another: &Self) -> bool {
        match self {
            Self::Param => another.is_param(),
            Self::Static{..} => self.to_static() == another.to_static(),
        }
    }

    fn into_radix(self) -> super::radix::Pattern {
        match self {
            Self::Param                        => super::radix::Pattern::Param,
            Self::Static(Cow::Borrowed(bytes)) => super::radix::Pattern::Static(bytes),
            Self::Static(Cow::Owned(vec))      => super::radix::Pattern::Static(vec.leak()),
        }
    }
}
