use crate::{
    __rt__,
    Status,
    Request,
    Context,
    Response,
    layer0_lib::{Method, Slice},
    layer3_fang_handler::{Handler, FrontFang, PathParams, BackFang},
};


/*===== defs =====*/
pub(crate) struct RadixRouter {
    pub(super) GET:    Node,
    pub(super) PUT:    Node,
    pub(super) POST:   Node,
    pub(super) PATCH:  Node,
    pub(super) DELETE: Node,
    pub(super) HEADfangs:    (&'static [FrontFang], &'static [BackFang]),
    pub(super) OPTIONSfangs: (&'static [FrontFang], &'static [BackFang]),
}

pub(super) struct Node {
    pub(super) patterns: &'static [Pattern],
    pub(super) front:    &'static [FrontFang],
    pub(super) handler:  Option<Handler>,
    pub(super) back:     &'static [BackFang],
    pub(super) children: Vec<Node>,
}

pub(super) enum Pattern {
    Static(&'static [u8]),
    Param,
} const _: () = {
    #[cfg(test)]
    impl std::fmt::Debug for Pattern {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Self::Param         => ":Param",
                Self::Static(bytes) => std::str::from_utf8(bytes).unwrap(),
            })
        }
    }
};


/*===== impls =====*/
impl RadixRouter {
    pub(crate) async fn handle(
        &self,
        mut c:   Context,
        mut req: Request,
        stream:  &mut __rt__::TcpStream,
    ) {
        let mut params = PathParams::new();
        let Some(target) = (match req.method() {
            Method::GET    => self.GET   .search(req.path_bytes(), &mut params),
            Method::PUT    => self.PUT   .search(req.path_bytes(), &mut params),
            Method::POST   => self.POST  .search(req.path_bytes(), &mut params),
            Method::PATCH  => self.PATCH .search(req.path_bytes(), &mut params),
            Method::DELETE => self.DELETE.search(req.path_bytes(), &mut params),
            
            Method::HEAD => {
                let (front, back) = self.HEADfangs;

                for ff in front {
                    (c, req) = match ff.0(c, req) {
                        Ok((c, req)) => (c, req),
                        Err(err_res) => return err_res.send(stream).await
                    }
                }

                let Some(target) = self.GET.search(req.path_bytes(), &mut params)
                    else {return c.NotFound().send(stream).await};
                
                let Response { headers, .. } = target.handle(c, req, params).await;
                let mut res = Response {
                    headers,
                    status:  Status::NoContent,
                    content: None,
                };

                for bf in back {
                    res = bf.0(res)
                }

                return res.send(stream).await
            }
            Method::OPTIONS => {
                let Some((cors_str, cors)) = crate::layer3_fang_handler::builtin::CORS.get() else {
                    return c.InternalServerError().send(stream).await
                };

                let (front, back) = self.OPTIONSfangs;

                for ff in front {
                    (c, req) = match ff.0(c, req) {
                        Ok((c, req)) => (c, req),
                        Err(err_res) => return err_res.send(stream).await
                    }
                }
                c.headers.Vary("Origin").cors(cors_str);

                {
                    let Some(origin) = req.header("Origin") else {
                        return c.BadRequest().send(stream).await
                    };
                    if !cors.AllowOrigin.matches(origin) {
                        return c.Forbidden().send(stream).await
                    }

                    if req.header("Authorization").is_some() && !cors.AllowCredentials {
                        return c.Forbidden().send(stream).await
                    }

                    if let Some(request_method) = req.header("Access-Control-Request-Method") {
                        let Some(allow_methods) = cors.AllowMethods.as_ref() else {
                            return c.Forbidden().send(stream).await
                        };

                        let request_method = Method::from_bytes(request_method.as_bytes());
                        if !allow_methods.contains(&request_method) {
                            return c.Forbidden().send(stream).await
                        }
                    }

                    if let Some(request_headers) = req.header("Access-Control-Request-Headers") {
                        let Some(allow_headers) = cors.AllowHeaders.as_ref() else {
                            return c.Forbidden().send(stream).await
                        };

                        let mut request_headers = request_headers.split(',').map(|h| h.trim_matches(' '));
                        if !request_headers.all(|h| allow_headers.contains(&h)) {
                            return c.Forbidden().send(stream).await
                        }
                    }
                }

                let mut res = c.NoContent().drop_content();

                for bf in back {
                    res = bf.0(res)
                }
                
                return res.send(stream).await
            }
        }) else {
            return c.NotFound().send(stream).await
        };

        target.handle(c, req, params).await.send(stream).await
    }
}

impl Node {
    #[inline] pub(super) async fn handle(&self,
        mut c:   Context,
        mut req: Request,
        params:  PathParams,
    ) -> Response {
        for f in self.front {
            (c, req) = match f.0(c, req) {
                Ok((c, req)) => (c, req),
                Err(err_res) => return err_res,
            }
        }
        match &self.handler {
            Some(h) => {
                let mut res = h.0(req, c, params).await;
                for b in self.back {
                    res = b.0(res);
                }
                res
            }
            None => c.NotFound()
        }
    }

    pub(super/* for test */) fn search(&self, mut path: &[u8], params: &mut PathParams) -> Option<&Node> {
        let mut target = self;
        loop {
            for pattern in target.patterns {
                if &path[0] == &b'/' {path = &path[1..]} else {
                    // At least one `pattern` to match is remaining
                    // but path doesn't start with '/'
                    return None
                }
                match pattern {
                    Pattern::Static(s) => path = path.strip_prefix(*s)?,
                    Pattern::Param => {
                        let (param, remaining) = split_next_section(path);
                        params.append(unsafe {Slice::from_bytes(param)});
                        path = remaining;
                    }
                }
            }

            if path.is_empty() {
                return Some(target)
            } else {
                target = target.matchable_child(path)?
            }
        }
    }
}


/*===== utils =====*/
impl Node {
    #[inline] fn matchable_child(&self, path: &[u8]) -> Option<&Node> {
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
            Self::Param     => true,
            Self::Static(s) => (&path[1..]/* skip initial '/' */).starts_with(s),
        }
    }
}

#[inline] fn split_next_section(path: &[u8]) -> (&[u8], &[u8]) {
    let len = path.len();
    let mut slash = len; for i in 0..len {
        if b'/' == path[i] {slash = i}
    }

    let after_slash = (slash + 1/* skip `/` */).min(len/* considering: `path` ends with `/` */);
    let ptr         = path.as_ptr();

    unsafe {(
        std::slice::from_raw_parts(ptr,                  slash),
        std::slice::from_raw_parts(ptr.add(after_slash), len - after_slash),
    )}
}
