use crate::{
    __dep__,
    Status,
    Request,
    Context,
    Response,
    layer0_lib::{Method},
    layer3_fang_handler::{Handler, FrontFang, PathParams, BackFang},
};


/*===== defs =====*/
pub(crate) struct RadixRouter {
    pub(super) GET:    Node,
    pub(super) PUT:    Node,
    pub(super) POST:   Node,
    pub(super) PATCH:  Node,
    pub(super) DELETE: Node,
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
        mut c: Context,
        req:   Request,
        mut stream: __dep__::TcpStream,
    ) {
        let path = req.path_bytes();
        let Some((target, params)) = (match req.method() {
            Method::GET     => self.GET   .search(path),
            Method::PUT     => self.PUT   .search(path),
            Method::POST    => self.POST  .search(path),
            Method::PATCH   => self.PATCH .search(path),
            Method::DELETE  => self.DELETE.search(path),
            Method::HEAD => {
                let Some((target, params)) = self.GET.search(path)
                    else {return c.NotFound().send(&mut stream).await};
                let Response { headers, .. } = target.handle(c, req, params).await;
                return Response {
                    headers,
                    status:  Status::NoContent,
                    content: None,
                }.send(&mut stream).await
            }
            Method::OPTIONS => {
                let Some(cors) = crate::CORS() else {
                    return c.InternalServerError().send(&mut stream).await
                };

                let headers = c.headers
                    .Vary("Origin")
                    .to_string();

                let send = |status: Status| Response {
                    status,
                    headers,
                    content: None,
                }.send(&mut stream);

                let Some(origin) = req.header("Origin") else {
                    return send(Status::BadRequest).await
                };
                if !cors.AllowOrigin.matches(origin) {
                    return send(Status::Forbidden).await
                }

                if req.header("Authorization").is_some() && !cors.AllowCredentials {
                    return send(Status::Forbidden).await
                }

                if let Some(request_method) = req.header("Access-Control-Request-Method") {
                    let request_method = Method::from_bytes(request_method.as_bytes());
                    match &cors.AllowMethods {
                        None => return send(Status::Forbidden).await,
                        Some(methods) => if !methods.contains(&request_method) {
                            return send(Status::Forbidden).await
                        }
                    }
                }

                if let Some(request_headers) = req.header("Access-Control-Request-Headers") {
                    let mut request_headers = request_headers.split(',').map(|h| h.trim_matches(' '));
                    match &cors.AllowHeaders {
                        None => return send(Status::Forbidden).await,
                        Some(headers) => if !request_headers.all(|h| headers.contains(&h)) {
                            return send(Status::Forbidden).await
                        }
                    }
                }
                
                return send(Status::NoContent).await
            }
        }) else {
            return c.NotFound().send(&mut stream).await
        };

        target.handle(c, req, params).await.send(&mut stream).await
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

    pub(super/* for test */) fn search(&self, mut path: &[u8]) -> Option<(&Node, PathParams)> {
        let mut path_len = path.len();
        if &path[path_len-1] == &b'/' {
            path = &path[..path_len-1];
            path_len -= 1;
        }

        let mut params = PathParams::new();
        let mut section_start = 1/* skip initial '/' */;

        let mut target = self;
        loop {
            for pattern in target.patterns {
                if &path[0] == &b'/' {path = &path[1..]} else {
                    // At least one `pattern` to match is remained
                    // but path doesn't start with '/'
                    return None
                }
                match pattern {
                    Pattern::Static(s) => {
                        path = path.strip_prefix(*s)?;
                        section_start += s.len() + 1/* skip '/' */;
                    }
                    Pattern::Param => match find(b'/', path) {
                        None => {
                            path = &[];
                            params.append(section_start..path_len)
                        }
                        Some(slash) => {
                            path = &path[slash+1..];
                            params.append(section_start..(section_start+slash));
                            section_start += slash + 1/* skip '/' */;
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

#[inline] fn find(b: u8, path: &[u8]) -> Option<usize> {
    for i in 0..(path.len()) {
        if b == path[i] {
            return Some(i)
        }
    }
    None
}
