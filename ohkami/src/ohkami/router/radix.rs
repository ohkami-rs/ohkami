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

#[derive(Debug)]
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
} const _: () = {
    struct C<Item: std::fmt::Debug>(Vec<Item>);
    impl<Item: std::fmt::Debug> FromIterator<Item> for C<Item> {
        fn from_iter<T: IntoIterator<Item = Item>>(iter: T) -> Self {
            Self(iter.into_iter().collect())
        }
    }
    impl<Item: std::fmt::Debug> std::fmt::Debug for C<Item> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_char('[')?;
            {
                let mut i = 0;
                loop {
                    if i == self.0.len() {break}

                    self.0[i].fmt(f)?;
                    i += 1;

                    if i < self.0.len()-1 {
                        f.write_char(',')?;
                        f.write_char(' ')?;
                    }
                }
            }
            f.write_char(']')?;
            Ok(())
        }
    }

    impl std::fmt::Debug for GlobalFangs {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut d = f.debug_struct("global_fangs");
            let mut d = &mut d;

            let mut set_once = false;
            if !(self.GET.0.is_empty() && self.GET.1.is_empty()) {set_once = true;
                d = d.field("GET",
                    &Iterator::chain(
                        self.GET.0.iter().map(|_| 'f'),
                        self.GET.1.iter().map(|_| 'f')
                    ).collect::<C<_>>()
                );
            }
            if !(self.PUT.0.is_empty() && self.PUT.1.is_empty()) {set_once = true;
                d = d.field("PUT",
                    &Iterator::chain(
                        self.PUT.0.iter().map(|_| 'f'),
                        self.PUT.1.iter().map(|_| 'f')
                    ).collect::<C<_>>()
                );
            }
            if !(self.POST.0.is_empty() && self.POST.1.is_empty()) {set_once = true;
                d = d.field("POST",
                    &Iterator::chain(
                        self.POST.0.iter().map(|_| 'f'),
                        self.POST.1.iter().map(|_| 'f')
                    ).collect::<C<_>>()
                );
            }
            if !(self.PATCH.0.is_empty() && self.PATCH.1.is_empty()) {set_once = true;
                d = d.field("PATCH",
                    &Iterator::chain(
                        self.PATCH.0.iter().map(|_| 'f'),
                        self.PATCH.1.iter().map(|_| 'f')
                    ).collect::<C<_>>()
                );
            }
            if !(self.DELETE.0.is_empty() && self.DELETE.1.is_empty()) {set_once = true;
                d = d.field("DELETE",
                    &Iterator::chain(
                        self.DELETE.0.iter().map(|_| 'f'),
                        self.DELETE.1.iter().map(|_| 'f')
                    ).collect::<C<_>>()
                );
            }
            if !(self.HEAD.0.is_empty() && self.HEAD.1.is_empty()) {set_once = true;
                d = d.field("HEAD",
                    &Iterator::chain(
                        self.HEAD.0.iter().map(|_| 'f'),
                        self.HEAD.1.iter().map(|_| 'f')
                    ).collect::<C<_>>()
                );
            }
            if !(self.OPTIONS.0.is_empty() && self.OPTIONS.1.is_empty()) {set_once = true;
                d = d.field("OPTIONS",
                    &Iterator::chain(
                        self.OPTIONS.0.iter().map(|_| 'f'),
                        self.OPTIONS.1.iter().map(|_| 'f')
                    ).collect::<C<_>>()
                );
            }

            if set_once {d.finish()} else {f.write_str(" {}")}
        }
    }
};

pub(super) struct Node {
    pub(super) patterns: &'static [Pattern],
    pub(super) handler:  Option<Handler>,
    pub(super) front:    &'static [FrontFang],
    pub(super) back:     &'static [BackFang],
    pub(super) children: Vec<Node>,
} const _: () = {
    impl std::fmt::Debug for Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            struct PatternsMarker(&'static [Pattern]);
            impl std::fmt::Debug for PatternsMarker {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_char('[')?;
                    'items: {
                        let mut n = self.0.len();
                        loop {
                            if n == 0 {break 'items}
                            f.write_str(&format!("{:?}", self.0[n-1]))?;
                            n -= 1;
                            if n > 1 {f.write_char(' ')?;}
                        }
                    }
                    f.write_char(']')?;

                    Ok(())
                }
            }

            enum HandlerMarker { None, Some }
            impl From<Option<&Handler>> for HandlerMarker {
                fn from(h: Option<&Handler>) -> Self {
                    match h {Some(_) => Self::Some, None => Self::None}
                }
            }
            impl std::fmt::Debug for HandlerMarker {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {Self::Some => f.write_char('@'), Self::None => f.write_str("None")}
                }
            }

            struct FangsMarker(usize);
            impl From<&[FrontFang]> for FangsMarker {
                fn from(ff: &[FrontFang]) -> Self {
                    Self(ff.len())
                }
            }
            impl From<&[BackFang]> for FangsMarker {
                fn from(bf: &[BackFang]) -> Self {
                    Self(bf.len())
                }
            }
            impl std::fmt::Debug for FangsMarker {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_char('[')?;
                    'items: {
                        let mut n = self.0;
                        loop {
                            if n == 0 {break 'items}
                            f.write_char('#')?;
                            n -= 1;
                            if n > 1 {
                                f.write_char(',')?;
                                f.write_char(' ')?;
                            }
                        }
                    }
                    f.write_char(']')?;

                    Ok(())
                }
            }

            f.debug_struct("")
                .field("patterns", &PatternsMarker(self.patterns))
                .field("handler", &HandlerMarker::from(self.handler.as_ref()))
                .field("front", &FangsMarker::from(self.front))
                .field("back", &FangsMarker::from(self.back))
                .field("children", &self.children)
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

                let mut res = handler.handle(req).await;  

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
            println!("[target] {:#?}", target);

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
                println!("not found, searching children: {:#?}", target.children);
        
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
