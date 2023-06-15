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
    pattern:  &'static [Pattern],
    front:    &'static [FrontFang],
    handler:  Option<Handler>,
    children: Vec<Node>,
}

enum Pattern {
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
            Method::GET => &self.GET,
            Method::PUT => &self.PUT,
            Method::POST => &self.POST,
            Method::HEAD => &self.HEAD,
            Method::PATCH => &self.PATCH,
            Method::DELETE => &self.DELETE,
            Method::OPTIONS => &self.OPTIONS,
        }.search(req.path()).await else {
            return Response::<()>::Err(c.NotFound()).send(&mut stream).await
        };

        for front in target.front {
            (c, req) = front(c, req).await;
        }

        // Here I'd like to write just
        // 
        // ```
        // let handler = unsafe{ target.handler.as_ref().unwrap_unchecked() };
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
        let /* mut */ res: Response = <Handler as Fn<(Request, Context, PathParams)>>::call(
            // SAFETY: `Node::search` returns Some(_) only when its `handler` is Some
            unsafe{ target.handler.as_ref().unwrap_unchecked() },
            (req, c, params)
        ).await;
        /*
            for back in target.back { ... }
        */

        res.send(&mut stream).await
    }
}

impl Node {
    async fn search(&self, path: &str) -> Option<(&Node, PathParams)> {
        let params = PathParams::new();

        let mut target = self;
        loop {
            todo!(TODO)
        }
    }
}
