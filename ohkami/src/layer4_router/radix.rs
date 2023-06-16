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
    GET: Node,
    PUT: Node,
    POST: Node,
    HEAD: Node,
    PATCH: Node,
    DELETE: Node,
    OPTIONS: Node,
}

struct Node {
    patterns: &'static [Pattern],
    front:    &'static [FrontFang],
    handler:  Option<Handler>,
    children: Vec<Node>,
}

enum Pattern {
    Static(&'static str),
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
        }.search(req.path()) else {
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
            None => {
                Response::<()>::Err(c.NotFound()).send(&mut stream).await
            }
        }
    }
}

impl Node {
    fn search(&self, mut path: &str) -> Option<(&Node, PathParams)> {
        let path_len = path.len();

        let mut params = PathParams::new();
        let mut param_start = 1/* skip initial '/' */;

        let mut target = self;
        loop {
            for pattern in target.patterns {
                path = path.strip_prefix('/')?;
                match pattern {
                    Pattern::Static(s) => {
                        path = path.strip_prefix(s)?;
                        param_start += s.len() + 1/* skip '/' */;
                    }
                    Pattern::Param => match path.find('/') {
                        None => {
                            path = "";
                            params.append(param_start..path_len)
                        }
                        Some(rem_len) => {
                            path = &path[rem_len+1..];
                            params.append(param_start..(param_start + rem_len));
                            param_start += rem_len + 1/* skip '/' */;
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
    #[inline(always)] fn matchable_child(&self, path: &str) -> Option<&Node> {
        for child in &self.children {
            if child.patterns.first()?.is_matchable_to(path) {
                return Some(child)
            }
        }
        None
    }
}

impl Pattern {
    #[inline(always)] fn is_matchable_to(&self, path: &str) -> bool {
        match self {
            Self::Param => true,
            Self::Static(s) => match path.find('/') {
                Some(slach) => &path[..slach] == *s,
                None        => path == *s,
            }
        }
    }
}
