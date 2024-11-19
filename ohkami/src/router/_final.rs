use super::_util;
use crate::fang::{FangProcCaller, BoxedFPC};
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

static NODE_BUFFER: std::sync::OnceLock<NodeBuffer> = std::sync::OnceLock::new();

struct NodeBuffer([MaybeUninit<Node>; NODE_BUFFER_SIZE]);
impl NodeBuffer {
    fn init(nodes: Vec<Node>) -> Self {
        if nodes.len() > NODE_BUFFER_SIZE {
            panic!("Currently, number of router nodes must be up to {NODE_BUFFER_SIZE}.")
        }

        let mut buf = [const {MaybeUninit::uninit()}; NODE_BUFFER_SIZE];
        for (i, node) in nodes.into_iter().enumerate() {
            buf[i].write(node);
        }

        Self(buf)
    }

    #[inline]
    unsafe fn get(&self, index: usize) -> &Node {
        self.0.get_unchecked(index).assume_init_ref()
    }

    #[inline]
    unsafe fn get_iter(&self, range: std::ops::Range<usize>) -> &[Node] {
        &*(self.0.get_unchecked(range) as *const [MaybeUninit<Node>] as *const [Node])
    }
}

pub(super) fn register_nodes(nodes: Vec<Node>) {
    NODE_BUFFER.set(NodeBuffer::init(nodes)).ok().expect("something went wrong around NODE_BUFFER");
}

#[allow(non_snake_case)]
fn NODES() -> &'static NodeBuffer {
    #[cfg(debug_assertions)]
    {NODE_BUFFER.get().expect("NODE_BUFFER is not initialized")}
    #[cfg(not(debug_assertions))]
    {unsafe {NODE_BUFFER.get().unwrap_unchecked()}}
}

const NODE_BUFFER_SIZE: usize = 128;

struct Node {
    pattern:  Pattern,
    proc:     BoxedFPC,
    catch:    BoxedFPC,
    children: &'static [Node],
}

enum Pattern {
    Static(&'static [u8]),
    Param,
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
