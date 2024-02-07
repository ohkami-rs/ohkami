use std::fmt::Write;

use crate::{
    Request,
    Response,
    Status,
    Method,
    IntoResponse,
    handler::Handler,
    fang::proc::{FrontFang, BackFang},
};
use ohkami_lib::{Slice, percent_decode};

#[cfg(feature="websocket")]
use crate::websocket::{
    UpgradeID,
    request_upgrade_id,
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
} const _: () = {
    impl std::fmt::Debug for Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            struct FangMarker;
            impl std::fmt::Debug for FangMarker {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_char('#')
                }
            }

            f.debug_struct("Node")
                .field("patterns", &self.patterns)
                .field("handler", &if self.handler.is_some() {"Some<_>"} else {"None"})
                .field("children", &self.children)
                .field("front", &self.front.iter().map(|_| FangMarker).collect::<Vec<_>>())
                .field("back", &self.back.iter().map(|_| FangMarker).collect::<Vec<_>>())
                .finish()
        }
    }
};

pub(super) enum Pattern {
    Static(&'static [u8]),
    Param,
} const _: () = {
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
// #[cfg(feature="websocket")] type HandleResult = (Response, Option<UpgradeID>);
// #[cfg(feature="websocket")] fn __no_upgrade(res: Response) -> HandleResult {
//     (res, None)
// }
// 
// #[cfg(not(feature="websocket"))] type HandleResult = Response;
// #[cfg(not(feature="websocket"))] fn __no_upgrade(res: Response) -> HandleResult {
//     res
// }


impl RadixRouter {
    pub(crate) async fn handle(
        &self,
        req: &mut Request,
    ) -> Response {
        let search_result = match req.method() {
            Method::GET    => self.GET   .search(req/*.path_bytes()*/),
            Method::PUT    => self.PUT   .search(req/*.path_bytes()*/),
            Method::POST   => self.POST  .search(req/*.path_bytes()*/),
            Method::PATCH  => self.PATCH .search(req/*.path_bytes()*/),
            Method::DELETE => self.DELETE.search(req/*.path_bytes()*/),
            
            Method::HEAD => {
                let (front, back) = self.HEADfangs;

                let mut res = 'res: {
                    for ff in front {
                        if let Err(err_res) = ff.0(req) {
                            break 'res err_res
                        }
                    }

                    let target = match self.GET.search(req/*.path_bytes()*/) {
                        Ok(Some(node)) => node,
                        Ok(None)       => break 'res Status::NotFound.into_response(),
                        Err(err_res)   => break 'res err_res,
                    };

                    let Response { headers, .. } = target.handle_discarding_upgrade(req).await;
                    Response {
                        headers,
                        status:  Status::NoContent,
                        content: None,
                    }
                };

                for bf in back {
                    if let Err(err_res) = bf.0(&mut res, req) {
                        return err_res;
                    }
                }

                return res;
            }

            Method::OPTIONS => {
                let (front, back) = self.OPTIONSfangs;

                let mut res = 'res: {
                    for ff in front {
                        if let Err(err_res) = ff.0(req) {
                            break 'res err_res
                        }
                    }
                    Status::NoContent.into_response()
                };

                for bf in back {
                    if let Err(err_res) = bf.0(&mut res, req) {
                        return err_res;
                    }
                }

                return res;
            }
        };

        match search_result {
            Ok(Some(node)) => node.handle(req).await,
            Ok(None)       => Status::NotFound.into_response(),
            Err(err_res)   => err_res,
        }
    }
}

impl Node {
    #[inline] pub(super) async fn handle(&self, req: &mut Request) -> Response {
        match &self.handler {
            Some(handler) => {
                #[cfg(feature="websocket")]
                let upgrade_id = match (handler.requires_upgrade).then(|| async {
                    let id = request_upgrade_id().await;
                    req.upgrade_id = Some(id);
                    id
                }) {None => None, Some(id) => Some(id.await)};

                let mut res = (handler.proc)(req).await;
                
                for bf in self.back {
                    if let Err(err_res) = bf.0(&mut res, req) {
                        return err_res;
                    }
                }

                #[cfg(feature="websocket")]
                {(res, upgrade_id)}
                #[cfg(not(feature="websocket"))]
                {res}
            }
            None => Status::NotFound.into_response(),
        }
    }

    #[inline] pub(super) async fn handle_discarding_upgrade(&self, req: &mut Request) -> Response {
        match &self.handler {
            Some(handler) => {
                let mut res = (handler.proc)(req).await;
                
                for bf in self.back {
                    if let Err(err_res) = bf.0(&mut res, req) {
                        return err_res;
                    }
                }

                res
            }
            None => Status::NotFound.into_response()
        }
    }

    pub(super/* for test */) fn search(&self, req: &mut Request) -> Result<Option<&Node>, Response> {
        let mut target = self;

        // SAFETY:
        // 1. `req` must be alive while `search`
        // 2. `Request` DOESN'T have method that mutates `path`,
        //    So what `path` refers to is NEVER changed by any other process
        //    while `search`
        let path_bytes_maybe_percent_encoded = unsafe {req.internal_path_bytes()};
        // Decode percent encodings in `path_bytes_maybe_percent_encoded`,
        // without checking entire it is valid UTF-8.
        let decoded = percent_decode(path_bytes_maybe_percent_encoded);
        let mut path: &[u8] = &decoded;

        #[cfg(feature="DEBUG")]
        println!("[path] '{}'", path.escape_ascii());

        loop {
            for ff in target.front {
                ff.0(req)?
            }

            #[cfg(feature="DEBUG")]
            println!("[patterns] {:?}", target.patterns);
    
            for pattern in target.patterns {
                if path.is_empty() || unsafe {path.get_unchecked(0)} != &b'/' {
                    // At least one `pattern` to match is remaining
                    // but remaining `path` doesn't start with '/'
                    return Ok(None)
                }

                path = unsafe {path.get_unchecked(1..)};
                
                #[cfg(feature="DEBUG")]
                println!("[path - prefix '/'] '{}'", path.escape_ascii());
        
                match pattern {
                    Pattern::Static(s)  => path = match path.strip_prefix(*s) {
                        Some(remaining) => remaining,
                        None            => return Ok(None),
                    },
                    Pattern::Param      => {
                        let (param, remaining) = split_next_section(path);
                        req.path.params.push(unsafe {Slice::from_bytes(param)});
                        path = remaining;
                    },
                }
            }

            if path.is_empty() {
                #[cfg(feature="DEBUG")]
                println!("Found: {target:?}");
        
                return Ok(Some(target))
            } else {
                #[cfg(feature="DEBUG")]
                println!("not found, searching children: {:?}", target.children);
        
                target = match target.matchable_child(path) {
                    Some(child) => child,
                    None        => return Ok(None),
                }
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

/// Returning `(next_section, remaining/* starts with '/', or empty */)`
#[inline] fn split_next_section(path: &[u8]) -> (&[u8], &[u8]) {
    let ptr = path.as_ptr();
    let len = path.len();

    for i in 0..len {
        if &b'/' == unsafe {path.get_unchecked(i)} {
            return unsafe {(
                std::slice::from_raw_parts(ptr,        i),
                std::slice::from_raw_parts(ptr.add(i), len - i),
            )}
        }
    } (path, &[])
}
