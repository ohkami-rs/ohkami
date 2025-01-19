use super::util::ID;
use super::segments::{RouteSegments, RouteSegment};
use crate::fang::{BoxedFPC, Fangs, handler::Handler};
use crate::ohkami::build::{ByAnother, HandlerSet};
use std::{sync::Arc, collections::HashSet};

#[cfg_attr(feature="openapi", derive(Clone))]
#[allow(non_snake_case)]
pub struct Router {
    id:     ID,
    routes: HashSet<&'static str>,
    pub(super) GET:     Node,
    pub(super) PUT:     Node,
    pub(super) POST:    Node,
    pub(super) PATCH:   Node,
    pub(super) DELETE:  Node,
    pub(super) OPTIONS: Node,
}

#[cfg_attr(feature="openapi", derive(Clone))]
pub(super) struct Node {
    pub(super) pattern:  Option<Pattern>,
    pub(super) handler:  Option<Handler>,
    pub(super) fangses:  FangsList,
    pub(super) children: Vec<Node>
}

#[derive(Clone)]
pub(super) enum Pattern {
    Static(&'static str),
    Param (&'static str)
}

#[derive(Clone)]
pub(super) struct FangsList(Vec<(
    ID,
    Arc<dyn Fangs>
)>);
impl FangsList {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add(&mut self, id: ID, fangs: Arc<dyn Fangs>) {
        if self.0.iter().find(|(_id, _)| *_id == id).is_none() {
            self.0.push((id, fangs));
        }
    }
    pub(super) fn extend(&mut self, another: Self) {
        for (id, fangs) in another.0.into_iter() {
            self.add(id, fangs)
        }
    }

    /// yield from most inner fangs
    fn into_iter(self) -> impl Iterator<Item = Arc<dyn Fangs>> {
        self.0.into_iter()
            .map(|(_, fangs)| fangs)
    }

    pub(super) fn into_proc_with(self, h: Handler) -> IntoProcWith {
        let mut iter = self.into_iter();

        #[cfg(not(feature="openapi"))]
        match iter.next() {
            None => h.proc,
            Some(most_inner) => iter.fold(
                most_inner.build(h.proc),
                |proc, fangs| fangs.build(proc)
            )
        }
        #[cfg(feature="openapi")]
        match iter.next() {
            None => (h.proc, h.openapi_operation),
            Some(most_inner) => iter.fold(
                (
                    most_inner.build(h.proc),
                    most_inner.openapi_map_operation(h.openapi_operation)
                ),
                |(proc, operation), fangs| (
                    fangs.build(proc),
                    fangs.openapi_map_operation(operation)
                )
            )
        }
    }
}
#[cfg(not(feature="openapi"))]
type IntoProcWith = BoxedFPC;
#[cfg(feature="openapi")]
type IntoProcWith = (BoxedFPC, crate::openapi::Operation);

impl Router {
    pub(crate) fn new() -> Self {
        Self {
            id:      ID::new(),
            routes:  HashSet::new(),
            GET:     Node::root(),
            PUT:     Node::root(),
            POST:    Node::root(),
            PATCH:   Node::root(),
            DELETE:  Node::root(),
            OPTIONS: Node::root(),
        }
    }

    pub(crate) fn id(&self) -> ID {
        self.id.clone()
    }

    pub(crate) fn register_handlers(&mut self, handlers: HandlerSet) {
        let HandlerSet { route, GET, PUT, POST, PATCH, DELETE } = handlers;

        self.routes.insert(route.literal());

        let methods = {
            macro_rules! allow_methods {
                ($($method:ident),*) => {{
                    let mut methods = Vec::new();
                    $(
                        if $method.is_some() {
                            methods.push(stringify!($method))
                        }
                    )*
                    methods
                }}
            }
            allow_methods! { GET, PUT, POST, PATCH, DELETE }
        };

        macro_rules! register {
            ($( $method:ident ),*) => {$(
                if let Some(h) = $method {
                    self.$method.register_handler(route.clone(), h, false).expect("Failed to register handler");
                }
            )*};
        }
        register! { GET, PUT, POST, PATCH, DELETE }

        self.OPTIONS.register_handler(route, Handler::default_options_with(methods), true).expect("Failed to register handler");
    }

    pub(crate) fn merge_another(&mut self, another: ByAnother) {
        let ByAnother { route, ohkami } = another;
        let another_routes = ohkami.into_router();

        crate::DEBUG!("merging following Ohkamis at {route:?}: \n\
            self: {self:#?}\n\
            another: {another_routes:#?}\n\
        ");

        macro_rules! merge {
            ($( $method:ident $( ( allow_override_handler = $allow_override_handler:literal ) )? ),*) => {
                $(
                    {
                        #[allow(unused_mut, unused_assignments)]
                        let mut allow_override_handler = false;
                        $( allow_override_handler = $allow_override_handler; )?

                        self.$method
                            .merge_node(
                                route.clone().into_iter(),
                                another_routes.$method,
                                allow_override_handler
                            )
                            .expect(&format!("Can't merge Ohkamis ({})", stringify!($method)));
                    }
                )*
            };
        }
        merge! {
            GET, PUT, POST, PATCH, DELETE,
            OPTIONS(allow_override_handler = true)
        }
    }

    pub(crate) fn apply_fangs(&mut self, id: ID, fangs: Arc<dyn Fangs>) {
        macro_rules! apply_to {
            ($($method:ident),*) => {
                $(
                    self.$method.apply_fangs(id.clone(), fangs.clone());
                )*
            };
        } apply_to! { GET, PUT, POST, PATCH, DELETE, OPTIONS }
    }

    #[allow(unused_mut)]
    pub(crate) fn finalize(mut self) -> (super::r#final::Router, HashSet<&'static str>) {
        let routes = std::mem::take(&mut self.routes);

        let r#final = super::r#final::Router::from(self);

        crate::DEBUG!("finalized: {final:#?}");

        (r#final, routes)
    }
}

impl Node {
    fn root() -> Self {
        Self {
            pattern:  None,
            handler:  None,
            fangses:  FangsList::new(),
            children: vec![],
        }
    }
    fn new(pattern: Pattern) -> Self {
        Self {
            pattern:  Some(pattern),
            handler:  None,
            fangses:  FangsList::new(),
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

    fn register_handler(
        &mut self,
        mut route:      RouteSegments,
        handler:        Handler,
        allow_override: bool,
    ) -> Result<(), String> {
        match route.next() {
            None => {
                self.set_handler(handler, allow_override)?;
                Ok(())
            }
            Some(segment) => {
                let pattern = Pattern::from(segment);
                match self.machable_child_mut(pattern.clone().into()) {
                    Some(child) => child.register_handler(route, handler, allow_override),
                    None => {
                        let mut child = Node::new(pattern.into());
                        child.register_handler(route, handler, allow_override)?;
                        self.append_child(child)?;
                        Ok(())
                    }
                }
            }
        }
    }

    fn append_child(&mut self, new_child: Node) -> Result<(), String> {
        match new_child.pattern.as_ref().expect("Invalid child node: Child node must have pattern") {
            Pattern::Param(_) => {
                self.children.push(new_child);
                Ok(())
            }
            Pattern::Static(s) => {
                if self.children.iter().find(|c|
                    c.pattern.as_ref().unwrap().to_static().is_some_and(|p| p == *s)
                ).is_some() {
                    let __position__ = match &self.pattern {
                        None    => format!("For the first part of route"),
                        Some(p) => format!("After {p:?}"),
                    };
                    Err(format!("Conflicting route definition: {__position__}, pattern '{s}' is registered twice"))
                } else {
                    self.children.push(new_child);
                    Ok(())
                }
            }
        }
    }

    fn append_fangs(&mut self, fangs: FangsList) {
        self.fangses.extend(fangs);
    }

    fn set_handler(&mut self, new_handler: Handler, allow_override: bool) -> Result<(), String> {
        if self.handler.is_some() && !allow_override {
            return Err(format!("Conflicting handler registering"))
        }
        self.handler = Some(new_handler);
        Ok(())
    }

    fn merge_node(
        &mut self,
        mut route_to_merge_root: RouteSegments,
        another: Node,
        allow_override_handler: bool
    ) -> Result<(), String> {
        match route_to_merge_root.next() {
            None => {
                self.merge_here(another, allow_override_handler)?;
                Ok(())
            }
            Some(pattern) => match self.machable_child_mut(pattern.clone().into()) {
                Some(child) => child.merge_node(route_to_merge_root, another, allow_override_handler),
                None => {
                    let mut new_child = Node::new(pattern.into());
                    new_child.merge_node(route_to_merge_root, another, allow_override_handler)?;
                    self.append_child(new_child)?;
                    Ok(())
                }
            }
        }
    }

    fn merge_here(&mut self, another_root: Node, allow_override_handler: bool) -> Result<(), String> {
        let Node {
            pattern:  None, /* another_root must be a root node and has pattern `None` */
            fangses:  another_root_fangses,
            handler:  another_root_handler,
            children: another_root_children,
        } = another_root else {
            panic!("Unexpectedly called `Node::merge_here` where `another_root` is not root node")
        };
        
        self.append_fangs(another_root_fangses);

        if let Some(h) = another_root_handler {
            self.set_handler(h, allow_override_handler)?;
        }

        for ac in another_root_children {
            self.append_child(ac)?
        }

        Ok(())
    }

    /// MUST be called after all handlers are registered
    fn apply_fangs(&mut self, id: ID, fangs: Arc<dyn Fangs>) {
        for child in &mut self.children {
            child.apply_fangs(id.clone(), fangs.clone())
        }

        self.fangses.add(id, fangs);
    }
}

impl Pattern {
    fn is_param(&self) -> bool {
        matches!(self, Pattern::Param { .. })
    }

    fn to_static(&self) -> Option<&str> {
        match self {
            Self::Param (_) => None,
            Self::Static(s) => Some(s)
        }
    }

    fn matches(&self, another: &Self) -> bool {
        match self {
            Self::Param (_) => another.is_param(),
            Self::Static(_) => self.to_static() == another.to_static(),
        }
    }

    #[cfg(feature="__rt_native__")]
    pub(super) fn is_static(&self) -> bool {
        matches!(self, Pattern::Static { .. })
    }

    #[cfg(feature="__rt_native__")]
    pub(super) fn merge_statics(self, child: Pattern) -> Option<Pattern> {
        match (self, child) {
            (Pattern::Static(s1), Pattern::Static(s2)) => Some(
                Pattern::Static([s1, s2].concat().leak())
            ),
            _ => None
        }
    }
}

const _: (/* conversions */) = {
    impl From<RouteSegment> for Pattern {
        fn from(segment: RouteSegment) -> Self {
            match segment {
                RouteSegment::Static(s)    => Self::Static(s),
                RouteSegment::Param (name) => Self::Param (name)
            }
        }
    }
};

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Param (name) => f.write_str(name),
            Self::Static(s)    => f.write_str(s),
        }
    }
}
#[cfg(feature="DEBUG")]
const _: (/* Debugs */) = {
    use super::util::{DebugSimpleIterator, DebugSimpleOption};

    impl std::fmt::Debug for Router {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BaseRouter")
                .field("GET", &self.GET)
                .field("PUT", &self.PUT)
                .field("POST", &self.POST)
                .field("PATCH", &self.PATCH)
                .field("DELETE", &self.DELETE)
                .field("OPTIONS", &self.OPTIONS)
                .field("id", &self.id)
                .field("routes", &self.routes)
                .finish()
        }
    }

    impl std::fmt::Debug for Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("")
                .field("pattern",  &DebugSimpleOption(&self.pattern))
                .field("handler",  &DebugSimpleOption(&self.handler))
                .field("fangs",    &self.fangses)
                .field("children", &self.children)
                .finish()
        }
    }

    impl std::fmt::Debug for FangsList {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            DebugSimpleIterator(self.0.iter().map(|(id, _)| id)).fmt(f)
        }
    }

};
