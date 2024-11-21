use super::{util, base};
use crate::fang::{FangProcCaller, BoxedFPC, Handler};
use crate::{request::Path, response::Content};
use crate::{Method, Request, Response};
use ohkami_lib::Slice;


#[derive(Debug)]
pub(crate) struct Router {
    GET:     Node,
    PUT:     Node,
    POST:    Node,
    PATCH:   Node,
    DELETE:  Node,
    OPTIONS: Node,
}

struct Node {
    pattern:  Pattern,
    proc:     BoxedFPC,
    catch:    BoxedFPC,
    children: &'static [Node]
}

#[derive(PartialEq)]
enum Pattern {
    Static(&'static [u8]),
    Param
}


impl Router {
    pub(crate) async fn handle(&self, req: &mut Request) -> Response {
        match req.method {
            Method::GET     => &self.GET,
            Method::PUT     => &self.PUT,
            Method::POST    => &self.POST,
            Method::PATCH   => &self.PATCH,
            Method::DELETE  => &self.DELETE,
            Method::OPTIONS => &self.OPTIONS,
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

        /*
            When `GET / HTTP/1.1` is coming, here
            
            * `bytes` is `b""` (by the normalization).
            * `self.pattern` is, an any pattern if compressed single-child,
              or `Static(b"")` by default.

            If compressed, the router has handlers only at or under the single pattern:

            ```
            /abc
            ├── /xyz  # /abc/xyz
            ├── /def  # /abc/def
            :
            ```

            and of course `self.pattern.take_through` returns `None`,
            `search` returns the catcher because no handler is registered to `/`.

            If not compressed, in other words, the router has multiple canidates for
            handler-routes under `/`:

            ```
            .             # /
            ├── /xyz      # /xyz
            │   ├── /pqr  # /xyz/pqr
            │   └── /def  # /xyz/def
            ├── /abc      # /abc
            :
            ```

            and `self.pattern.take_through` successes by matching `b""`,
            then `search` returns the proc (the handler if user registered, or
            NotFound handler if not, with fangs).
            When `GET /abc HTTP/1.1` is coming, this `self.pattern.take_through`
            successes with `Some(b"/abc")`, then we just perform `bytes = b"/abc"`
            and go to `'next_target` loop.
        */
        if let Some(remaining) = self.pattern.take_through(bytes, path) {
            if remaining.is_empty() {
                return &self.proc
            } else {
                bytes = remaining
            }
        } else {
            return &self.catch
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
        crate::DEBUG!("[Pattern::take_through] self: `{self:?}`, bytes: '{}'", bytes.escape_ascii());
        match self {
            Pattern::Static(s) => {
                let size = s.len();
                if bytes.len() >= size && *s == unsafe {bytes.get_unchecked(..size)} {
                    crate::DEBUG!("[Pattern::take_through] Static => remaining = Some('{}')", bytes[size..].escape_ascii());
                    Some(unsafe {bytes.get_unchecked(size..)})
                } else {
                    crate::DEBUG!("[Pattern::take_through] Static => remaining = None");
                    None
                }
            }
            Pattern::Param => {
                if bytes.len() >= 2
                && *unsafe {bytes.get_unchecked(0)} == b'/'
                && *unsafe {bytes.get_unchecked(1)} != b'/' {
                    let (param, remaining) = util::split_next_section(unsafe {bytes.get_unchecked(1..)});
                    unsafe {path.push_param(Slice::from_bytes(param))};
                    crate::DEBUG!("[Pattern::take_through] Param => remaining = Some('{}')", remaining.escape_ascii());
                    Some(remaining)
                } else {
                    crate::DEBUG!("[Pattern::take_through] Param => remaining = None");
                    None
                }
            }
        }
    }
}


const _: (/* conversions */) = {
    static mut STACK: NodeStack = NodeStack::new();

    impl From<base::Router> for Router {
        fn from(base: base::Router) -> Self {
            Router {
                GET:     Node::from(base.GET),
                PUT:     Node::from(base.PUT),
                POST:    Node::from(base.POST),
                PATCH:   Node::from(base.PATCH),
                DELETE:  Node::from(base.DELETE),
                OPTIONS: Node::from(base.OPTIONS),
            }
        }
    }
    
    impl From<base::Node> for Node {
        fn from(mut base: base::Node) -> Self {
            /* merge single-child static pattern and compress routing tree */
            while base.children.len() == 1
            && base.handler.is_none()
            && base.pattern.as_ref().is_none_or(|p| p.is_static())
            && base.children[0].pattern.as_ref().unwrap(/* not root */).is_static() {
                let child = base.children.pop().unwrap(/* base.children.len() == 1 */);
                base.children = child.children;
                base.handler = child.handler;
                base.fangses.extend(child.fangses);
                base.pattern = Some(match base.pattern {
                    None    => child.pattern.unwrap(/* not root */),
                    Some(p) => p.merge_statics(child.pattern.unwrap(/* not root */)).unwrap(/* both are Pattern::Static */)
                });
            }

            let stack = unsafe {#[allow(static_mut_refs)] &mut STACK};

            let mut direct_children = Vec::with_capacity(base.children.len());
            for child in base.children {
                direct_children.push(Node::from(child));
            }

            direct_children.sort_by(|a, b| match (&a.pattern, &b.pattern) {
                (Pattern::Static(a), Pattern::Static(b)) => a.cmp(b).reverse(),
                (Pattern::Static(_), Pattern::Param)     => std::cmp::Ordering::Less,
                (Pattern::Param,     Pattern::Static(_)) => std::cmp::Ordering::Greater,
                _                                        => std::cmp::Ordering::Equal
            });

            let n_children = direct_children.len();
            for child in direct_children {
                stack.push(child);
            } /* here last `n_children` elements of `stack` are THE `children` */
            Node {
                pattern:  base.pattern.map(Pattern::from).unwrap_or(Pattern::Static(b"")),
                proc:     base.fangses.clone().into_proc_with(base.handler.unwrap_or(Handler::default_not_found())),
                catch:    base.fangses.into_proc_with(Handler::default_not_found()),
                children: stack.slice((stack.next - n_children)..(stack.next))
            }
        }
    }

    impl From<base::Pattern> for Pattern {
        fn from(base: base::Pattern) -> Self {
            match base {
                base::Pattern::Static { route, range } => Self::Static(route[range].as_bytes()),
                base::Pattern::Param  { .. }           => Self::Param
            }
        }
    }

    const NODE_STACK_SIZE: usize = 256;
    struct NodeStack {
        nodes: [std::mem::MaybeUninit<Node>; NODE_STACK_SIZE],
        next:  usize,
    }
    impl NodeStack {
        const fn new() -> Self {
            Self {
                nodes: [const {std::mem::MaybeUninit::uninit()}; 256],
                next:  0
            }
        }

        fn push(&mut self, node: Node) {
            if self.next == NODE_STACK_SIZE {
                panic!("NodeStack is already full!")
            }
            self.nodes[self.next].write(node);
            self.next += 1;
        }

        fn slice(&self, range: std::ops::Range<usize>) -> &[Node] {
            #[cfg(debug_assertions)] {
                assert!(range.end <= self.next)
            }
            unsafe {&*(self.nodes.get(range).unwrap() as *const [std::mem::MaybeUninit<Node>] as *const [Node])}
        }
    }
};

const _: (/* Debugs */) = {
    impl std::fmt::Debug for Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("")
                .field("pattern",  &self.pattern)
                .field("children", &self.children)
                .finish()
        }
    }

    impl std::fmt::Debug for Pattern {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Self::Param     => ":param",
                Self::Static(s) => std::str::from_utf8(s).unwrap(),
            })
        }
    }
};
