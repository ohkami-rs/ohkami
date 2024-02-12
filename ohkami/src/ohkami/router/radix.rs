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
    pub(super) global_fangs: GlobalFangs,
    pub(super) GET:          Node,
    pub(super) PUT:          Node,
    pub(super) POST:         Node,
    pub(super) PATCH:        Node,
    pub(super) DELETE:       Node,
}

pub(super) struct GlobalFangs {
    pub(super) GET:     (&'static [FrontFang], &'static [BackFang]),
    pub(super) PUT:     (&'static [FrontFang], &'static [BackFang]),
    pub(super) POST:    (&'static [FrontFang], &'static [BackFang]),
    pub(super) PATCH:   (&'static [FrontFang], &'static [BackFang]),
    pub(super) DELETE:  (&'static [FrontFang], &'static [BackFang]),
    pub(super) HEAD:    (&'static [FrontFang], &'static [BackFang]),
    pub(super) OPTIONS: (&'static [FrontFang], &'static [BackFang]),
}

pub(super) struct Node {
    pub(super) patterns: &'static [Pattern],
    pub(super) handler:  Option<Handler>,
    pub(super) front:    &'static [FrontFang],
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

impl RadixRouter {
    pub(crate) async fn handle(
        &self,
        req: &mut Request,
    ) -> Response {
        let method = req.method();

        let (global_front, global_back) = match &method {
            Method::GET     => self.global_fangs.GET,
            Method::PUT     => self.global_fangs.PUT,
            Method::POST    => self.global_fangs.POST,
            Method::PATCH   => self.global_fangs.PATCH,
            Method::DELETE  => self.global_fangs.DELETE,
            Method::HEAD    => self.global_fangs.HEAD,
            Method::OPTIONS => self.global_fangs.OPTIONS,
        };

        let mut res = 'handled: {
            for gf in global_front {
                if let Err(err_res) = gf.call(req).await {
                    break 'handled err_res
                }
            }

            let search_result = match method {
                Method::GET     => self.GET   .search(req),
                Method::PUT     => self.PUT   .search(req),
                Method::POST    => self.POST  .search(req),
                Method::PATCH   => self.PATCH .search(req),
                Method::DELETE  => self.DELETE.search(req),
                Method::OPTIONS => break 'handled Response::NoContent(),
                Method::HEAD    => break 'handled match self.GET.search(req) {
                    Some(n) => n.handle(req).await.without_content(),
                    None    => Response::NotFound(),
                },
            };

            match search_result {
                Some(n) => n.handle(req).await,
                None    => Status::NotFound.into_response(),
            }
        };

        for gb in global_back {
            if let Err(err_res) = gb.call(&mut res, req).await {
                return err_res
            }
        }

        res
    }
}

impl Node {
    #[inline] pub(super) async fn handle(&self, req: &mut Request) -> Response {
        match &self.handler {
            Some(handler) => {
                for ff in self.front {
                    if let Err(err_res) = ff.call(req).await {
                        return err_res;
                    }
                }

                let mut res = (handler.proc)(req).await;  

                for bf in self.back {
                    if let Err(err_res) = bf.call(&mut res, req).await {
                        return err_res;
                    }
                }
                res
            }
            None => Status::NotFound.into_response(),
        }
    }

    pub(super/* for test */) fn search(&self, req: &mut Request) -> Option<&Node> {
        let mut target = self;

        // SAFETY:
        // 1. `req` must be alive while `search`
        // 2. `Request` DOESN'T have method that mutates `path`,
        //    So what `path` refers to is NEVER changed by any other process
        //    while `search`
        let path_bytes_maybe_percent_encoded = unsafe {req.internal_path_bytes()};
        // Decode percent encodings in `path_bytes_maybe_percent_encoded`,
        // without checking if entire it is valid UTF-8.
        let decoded = percent_decode(path_bytes_maybe_percent_encoded);
        let mut path: &[u8] = &decoded;

        #[cfg(feature="DEBUG")]
        println!("[path] '{}'", path.escape_ascii());

        loop {
            #[cfg(feature="DEBUG")]
            println!("[target] {:?}", target);

            #[cfg(feature="DEBUG")]
            println!("[patterns] {:?}", target.patterns);
    
            for pattern in target.patterns {
                if path.is_empty() || unsafe {path.get_unchecked(0)} != &b'/' {
                    // At least one `pattern` to match is remaining
                    // but remaining `path` doesn't start with '/'
                    return None
                }

                path = unsafe {path.get_unchecked(1..)};
                
                #[cfg(feature="DEBUG")]
                println!("[path striped prefix '/'] '{}'", path.escape_ascii());
        
                match pattern {
                    Pattern::Static(s)  => path = match path.strip_prefix(*s) {
                        Some(remaining) => remaining,
                        None            => return None,
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
        
                return Some(target)
            } else {
                #[cfg(feature="DEBUG")]
                println!("not found, searching children: {:?}", target.children);
        
                target = match target.matchable_child(path) {
                    Some(child) => child,
                    None        => return None,
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
