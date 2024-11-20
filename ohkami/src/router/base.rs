use super::util::ID;
use super::segments::{RouteSegments, RouteSegment};
use crate::fang::{BoxedFPC, Fangs, Handler};
use crate::ohkami::build::HandlerSet;
use std::{sync::Arc, ops::Range, collections::HashSet};


pub(crate) struct Router {
    id:     ID,
    routes: HashSet<&'static str>,
    pub(super) GET:     Node,
    pub(super) PUT:     Node,
    pub(super) POST:    Node,
    pub(super) PATCH:   Node,
    pub(super) DELETE:  Node,
    pub(super) OPTIONS: Node,
}

pub(super) struct Node {
    pub(super) pattern:  Option<Pattern>,
    pub(super) handler:  Option<Handler>,
    pub(super) fangses:  FangsList,
    pub(super) children: Vec<Node>
}

#[derive(Clone)]
pub(super) enum Pattern {
    Static { route: &'static str, range: Range<usize> },
    Param
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

    pub(super) fn into_proc_with(self, handler: Handler) -> BoxedFPC {
        let mut iter = self.into_iter();

        match iter.next() {
            None => handler.into(),
            Some(most_inner) => iter.fold(
                most_inner.build(handler.into()),
                |proc, fangs| fangs.build(proc)
            )
        }
    }

    /// yield from most inner fangs
    fn iter(&self) -> impl Iterator<Item = &Arc<dyn Fangs>> {
        self.0.iter()
            .map(|(_, fangs)| fangs)
    }
    /// yield from most inner fangs
    fn into_iter(self) -> impl Iterator<Item = Arc<dyn Fangs>> {
        self.0.into_iter()
            .map(|(_, fangs)| fangs)
    }
}

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

        let methods = if !self.routes.insert(route.literal()) {
            panic!("Duplicate routes registration: `{}`", route.literal())
        } else {
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
            } allow_methods! { GET, PUT, POST, PATCH, DELETE }
        };

        macro_rules! register {
            ($( $method:ident ),*) => {$(
                if let Some(h) = $method {
                    self.$method.register_handler(route.clone(), h).expect("Failed to register handler");
                }
            )*};
        } register! { GET, PUT, POST, PATCH, DELETE }

        self.OPTIONS.register_handler(route, Handler::new(move |req| {
            let mut available_methods = methods.clone();
            if available_methods.contains(&"GET") {
                available_methods.push("HEAD")
            }
            available_methods.push("OPTIONS");

            Box::pin(async move {
                #[cfg(debug_assertions)] {
                    assert_eq!(req.method, crate::Method::OPTIONS);
                }

                match req.headers.AccessControlRequestMethod() {
                    Some(method) => {
                        /*
                            Ohkami, by default, does nothing more than setting
                            `Access-Control-Allow-Methods` to preflight request.
                            CORS fang must override `Not Implemented` response,
                            whitch is the default for a valid preflight request,
                            by a successful one in its proc.
                        */
                        (if available_methods.contains(&method) {
                            crate::Response::NotImplemented()
                        } else {
                            crate::Response::BadRequest()
                        }).with_headers(|h| h
                            .AccessControlAllowMethods(available_methods.join(", "))
                        )
                    }
                    None => {
                        /*
                            For security reasons, Ohkami doesn't support the
                            normal behavior to OPTIONS request like

                            ```
                            crate::Response::NoContent()
                                .with_headers(|h| h
                                    .Allow(available_methods.join(", "))
                                )
                            ```
                        */
                        crate::Response::NotFound()
                    }
                }
            })
        })).expect("Failed to register handler")
    }

    pub(crate) fn finalize(self) -> super::r#final::Router {
        super::r#final::Router::from(self)
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
        mut route: RouteSegments,
        handler:   Handler,
    ) -> Result<(), String> {
        match route.next() {
            None => {
                self.set_handler(handler)?;
                Ok(())
            }
            Some(segment) => {
                let pattern = Pattern::from(segment);
                match self.machable_child_mut(pattern.clone().into()) {
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
    }

    fn append_child(&mut self, new_child: Node) -> Result<(), String> {
        match new_child.pattern.as_ref().expect("Invalid child node: Child node must have pattern") {
            Pattern::Param => {
                self.children.push(new_child);
                Ok(())
            }
            Pattern::Static { route, range } => {
                let s = &route[range.clone()];
                if self.children.iter().find(|c|
                    c.pattern.as_ref().unwrap().to_static().is_some_and(|p| p == s)
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

    fn set_handler(&mut self, new_handler: Handler) -> Result<(), String> {
        self.handler.is_none()
            .then(|| self.handler = Some(new_handler))
            .ok_or_else(|| format!("Conflicting handler registering"))
    }
}

impl Pattern {
    pub(super) fn is_static(&self) -> bool {
        matches!(self, Pattern::Static { .. })
    }

    pub(super) fn merge_statics(self, child: Pattern) -> Option<Pattern> {
        match (self, child) {
            (
                Pattern::Static { route: this_route,  range: this_range },
                Pattern::Static { route: child_route, range: child_range }
            ) => {
                if &child_route[(child_range.start - this_range.len())..(child_range.start)]
                == &this_route[this_range.start..this_range.end] {                    
                    Some(Pattern::Static {
                        route: child_route,
                        range: (child_range.start - this_range.len())..(child_range.end)
                    })
                } else {
                    None
                }
            }
            _ => None
        }
    }

    fn is_param(&self) -> bool {
        matches!(self, Pattern::Param)
    }

    fn to_static(&self) -> Option<&str> {
        match self {
            Self::Param                   => None,
            Self::Static { route, range } => Some(&route[range.clone()])
        }
    }

    fn matches(&self, another: &Self) -> bool {
        match self {
            Self::Param => another.is_param(),
            Self::Static{..} => self.to_static() == another.to_static(),
        }
    }
}

impl From<RouteSegment> for Pattern {
    fn from(segment: RouteSegment) -> Self {
        match segment {
            RouteSegment::Static { route, range } => Self::Static { route, range },
            RouteSegment::Param                   => Self::Param
        }
    }
}

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Param                   => f.write_str(":Param"),
            Self::Static { route, range } => f.write_str(&route[range.clone()]),
        }
    }
}
