use crate::layer3_fang_handler::{Handler, FrontFang, Handlers};
type Range = std::ops::Range<usize>;


/*===== defs =====*/
pub(crate) struct TrieRouter {
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
    fangs:    Vec<FrontFang>,
    handler:  Option<Handler>,
    children: Vec<Node>,
}

enum Pattern {
    Static{route: &'static [u8], range: Range},
    Param,
}


/*===== public impls =====*/
impl TrieRouter {
    pub(crate) fn new() -> Self {
        Self {
            GET: Node::root(),
            PUT: Node::root(),
            POST: Node::root(),
            HEAD: Node::root(),
            PATCH: Node::root(),
            DELETE: Node::root(),
            OPTIONS: Node::root(),
        }
    }

    pub(crate) fn register_handlers(&mut self, handlers: Handlers) {
        let Handlers { route, GET, PUT, POST, HEAD, PATCH, DELETE, OPTIONS } = handlers;

        compile_error!(TODO)
    }
}


/*===== internal impls =====*/
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
