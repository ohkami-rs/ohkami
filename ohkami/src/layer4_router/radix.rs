use crate::{
    __dep__,
    Request,
    Context,
    Response,
    layer0_lib::{Method},
    layer3_fang_handler::{Handler, FrontFang, PathParams},
};


/*===== defs =====*/
pub(crate) struct RadixRouter {
    pub(super) GET: Node,
    pub(super) PUT: Node,
    pub(super) POST: Node,
    pub(super) HEAD: Node,
    pub(super) PATCH: Node,
    pub(super) DELETE: Node,
    pub(super) OPTIONS: Node,
}

pub(super) struct Node {
    pub(super) patterns: &'static [Pattern],
    pub(super) front:    &'static [FrontFang],
    pub(super) handler:  Option<Handler>,
    pub(super) children: Vec<Node>,
}

pub(super) enum Pattern {
    Static(&'static [u8]),
    Param,
}


/*===== impls =====*/
impl RadixRouter {
    pub(crate) async fn handle(
        &self,
        mut c: Context,
        mut req: Request,
        mut stream: __dep__::TcpStream,
    ) {
        let Some((target, params)) = match req.method() {
            Method::GET     => &self.GET,
            Method::PUT     => &self.PUT,
            Method::POST    => &self.POST,
            Method::HEAD    => &self.HEAD,
            Method::PATCH   => &self.PATCH,
            Method::DELETE  => &self.DELETE,
            Method::OPTIONS => &self.OPTIONS,
        }.search(req.path_bytes()) else {
            return Response::<()>::Err(c.NotFound()).send(&mut stream).await
        };

        match &target.handler {
            Some(handler) => {
                for front in target.front {
                    (c, req) = front(c, req).await;
                }

                // Here I'd like to write just
                // 
                // ```
                // let res = handler(req, c, params) ...
                // ```
                // 
                // but this causes annoying panic for rust-analyzer (v0.3.1549).
                // 
                // Based on the logs, it seems that:
                // 
                // 1. This `handler` is `&Handler` and I meen `handler(...)` is
                //    calling `<Handler as **Fn**>::call`.
                // 2. But rust-analyzer thinks this `handler(...)` is calling
                //    `<Handler as **FnOnce**>::call_once`.
                // 
                // So I explicitly indicate 1. (This may be fixed in future)
                let /* mut */ res: Response = <Handler as Fn<(Request, Context, PathParams)>>::call(handler, (req, c, params)).await;

                /*
                for back in target.back {
                    res = back(res).await;
                }
                */

                res.send(&mut stream).await
            }
            None => Response::<()>::Err(c.NotFound()).send(&mut stream).await
        }
    }
}

impl Node {
    fn search(&self, mut path: &[u8]) -> Option<(&Node, PathParams)> {
        let path_len = path.len();

        let mut params = PathParams::new();
        let mut param_start = 1/* skip initial '/' */;

        let mut target = self;
        loop {
            for pattern in target.patterns {
                path = path.strip_prefix(&[b'/'])?;
                match pattern {
                    Pattern::Static(s) => {
                        path = path.strip_prefix(*s)?;
                        param_start += s.len() + 1/* skip '/' */;
                    }
                    Pattern::Param => match find(b'/', path) {
                        None => {
                            path = &[];
                            params.append(param_start..path_len)
                        }
                        Some(slash) => {
                            path = &path[slash+1..];
                            params.append(param_start..(param_start + slash));
                            param_start += slash + 1/* skip '/' */;
                        }
                    } 
                }
            }

            if path.is_empty() {
                return Some((target, params))
            } else {
                target = target.matchable_child(path)?
            }
        }
    }
}


/*===== utils =====*/
impl Node {
    #[inline(always)] fn matchable_child(&self, path: &[u8]) -> Option<&Node> {
        for child in &self.children {
            if child.patterns.first()?.is_matchable_to(path) {
                return Some(child)
            }
        }
        None
    }
}

impl Pattern {
    #[inline(always)] fn is_matchable_to(&self, path: &[u8]) -> bool {
        match self {
            Self::Param => true,
            Self::Static(s) => match find(b'/', path) {
                Some(slach) => &path[..slach] == *s,
                None        => path == *s,
            }
        }
    }
}

#[inline(always)] fn find(b: u8, path: &[u8]) -> Option<usize> {
    let mut index = None;
    for i in 0..(path.len()) {
        if path[i] == b {
            index = Some(i);
            break
        }
    }
    index
}
