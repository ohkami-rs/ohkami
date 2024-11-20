use super::{_util, _base};
use crate::fang::{FangProcCaller, BoxedFPC, Handler};
use crate::{request::Path, response::Content};
use crate::{Method, Request, Response};
use ohkami_lib::Slice;
use std::mem::MaybeUninit;


pub(crate) struct Router {
    GET:     &'static Node,
    PUT:     &'static Node,
    POST:    &'static Node,
    PATCH:   &'static Node,
    DELETE:  &'static Node,
    OPTIONS: &'static Node,
}

struct Node {
    pattern:  Pattern,
    proc:     BoxedFPC,
    catch:    BoxedFPC,
    children: &'static [Node]
}

enum Pattern {
    Static(&'static [u8]),
    Param
}


impl Router {
    pub(crate) async fn handle(&self, req: &mut Request) -> Response {
        match req.method {
            Method::GET     => self.GET,
            Method::PUT     => self.PUT,
            Method::POST    => self.POST,
            Method::PATCH   => self.PATCH,
            Method::DELETE  => self.DELETE,
            Method::OPTIONS => self.OPTIONS,
            Method::HEAD => return {
                let mut res = self.GET.search(&mut req.path).call_bite(req).await;
                {/* not `res.drop_content()` to leave `Content-Type`, `Content-Length` */
                    res.content = Content::None;
                }
                res
            }
        }.search(&mut req.path).call_bite(req).await
    }
}

impl Node {
    /// ## Precondition
    /// 
    /// `patterns`s of all `Node`s belonging to this tree MUST be:
    /// 
    /// 1. all `Pattern::Static`s are sorted in reversed alphabetical order
    /// 2. zero or one `Pattern::Param` exists at the end
    fn search(&self, path: &mut Path) -> &dyn FangProcCaller {
        let mut bytes = unsafe {path.normalized_bytes()};

        if bytes.is_empty() {/* e.g. GET / HTTP/1.1 */
            return &self.proc/* proc registered to the root Node */
        }

        let mut target = self;
        'next_target: loop {
            for child in target.children {
                if let Some(remaining) = child.pattern.take_through(bytes, path) {
                    if remaining.is_empty() {
                        return &child.proc
                    } else {
                        bytes  = remaining;
                        target = child;
                        continue 'next_target
                    }
                }
            }; return &target.catch
        }
    }
}

impl Pattern {
    /// ## Precondition
    /// 
    /// `self`, if `Static`, must hold bytes starting with `/` e.g. `/abc`, `/`, `/abc/xyz`
    #[inline]
    fn take_through<'b>(
        &self,
        bytes: &'b [u8],
        path:  &mut Path
    ) -> Option<&'b [u8]/* remaining part of `bytes` */> {
        match self {
            Pattern::Static(s) => {
                let size = s.len();
                if bytes.len() >= size && *s == unsafe {bytes.get_unchecked(..size)} {
                    Some(unsafe {bytes.get_unchecked(size..)})
                } else {
                    None
                }
            }
            Pattern::Param => {
                if bytes.len() >= 2
                && *unsafe {bytes.get_unchecked(0)} == b'/'
                && *unsafe {bytes.get_unchecked(1)} != b'/' {
                    let (param, remaining) = _util::split_next_section(unsafe {bytes.get_unchecked(1..)})?;
                    unsafe {path.push_param(Slice::from_bytes(param))};
                    Some(remaining)
                } else {
                    None
                }
            }
        }
    }
}


const _: () = {
    static mut NODES: NodeBuffer = NodeBuffer::new();

    impl From<_base::Router> for Router {
        fn from(base: _base::Router) -> Self {
            fn registered(node: Node) -> &'static Node {
                let nodes = unsafe {#[allow(static_mut_refs)] &mut NODES};
                nodes.push(node);
                nodes.get(nodes.next - 1)
            }
            Router {
                GET:     registered(Node::from(base.GET)),
                PUT:     registered(Node::from(base.PUT)),
                POST:    registered(Node::from(base.POST)),
                PATCH:   registered(Node::from(base.PATCH)),
                DELETE:  registered(Node::from(base.DELETE)),
                OPTIONS: registered(Node::from(base.OPTIONS)),
            }
        }
    }
    
    impl From<_base::Node> for Node {
        fn from(mut base: _base::Node) -> Self {
            /* merge single-child static patterns to compress routing tree */
            while base.children.len() == 1
            && base.handler.is_none()
            && base.pattern.as_ref().is_some_and(|p| p.is_static())
            && base.children[0].pattern.as_ref().unwrap(/* not root */).is_static() {
                let child = base.children.pop().unwrap(/* checked: base.children.len() == 1 */);
                base.children = child.children;
                base.handler  = child.handler;
                base.fangses.extend(child.fangses);
                base.pattern.replace(_base::Pattern::merge_statics(
                    base.pattern.clone().unwrap(/* checked: base.pattern.as_ref().is_some_and(..) */),
                    child.pattern.unwrap(/* not root */)
                ).unwrap(/* both are Pattern::Static */));
            }

            let nodes = unsafe {#[allow(static_mut_refs)] &mut NODES};

            let n_chilren = base.children.len();
            for child in base.children {
                nodes.push(Node::from(child));
            }
            
            Node {
                pattern:  base.pattern.map(Pattern::from).unwrap_or(Pattern::Static(&[])),
                proc:     base.fangses.clone().into_proc_with(base.handler.unwrap_or(Handler::default_not_found())),
                catch:    base.fangses.into_proc_with(Handler::default_not_found()),
                children: nodes.slice((nodes.next - n_chilren)..(nodes.next))
            }
        }
    }

    impl From<_base::Pattern> for Pattern {
        fn from(base: _base::Pattern) -> Self {
            match base {
                _base::Pattern::Static { route, range } => Self::Static(route[range].as_bytes()),
                _base::Pattern::Param                   => Self::Param
            }
        }
    }

    const NODE_BUFFER_SIZE: usize = 256;
    struct NodeBuffer {
        nodes: [std::mem::MaybeUninit<Node>; NODE_BUFFER_SIZE],
        next:  usize,
    }
    impl NodeBuffer {
        const fn new() -> Self {
            Self {
                nodes: [const {std::mem::MaybeUninit::uninit()}; 256],
                next:  0
            }
        }
        fn push(&mut self, node: Node) {
            if self.next == NODE_BUFFER_SIZE {
                panic!("NodeBuffer is already full!")
            }
            self.nodes[self.next].write(node);
            self.next += 1;
        }
        fn get(&self, index: usize) -> &Node {
            #[cfg(debug_assertions)] {
                assert!(index < self.next)
            }
            unsafe {self.nodes.get_unchecked(index).assume_init_ref()}
        }
        fn slice(&self, range: std::ops::Range<usize>) -> &[Node] {
            #[cfg(debug_assertions)] {
                assert!(range.end < self.next)
            }
            unsafe {&*(self.nodes.get_unchecked(range) as *const [std::mem::MaybeUninit<Node>] as *const [Node])}
        }
    }
};
