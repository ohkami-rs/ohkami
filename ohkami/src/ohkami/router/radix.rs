use crate::{handler::Handler, Method, Request, Response};
use crate::fang::{FangProcCaller, BoxedFPC};
use ohkami_lib::Slice;
use std::fmt::Write as _;


#[derive(Debug)]
pub(crate) struct RadixRouter(Node);

pub(super) struct Node {
    pub(super) patterns: &'static [Pattern],
    pub(super) proc:     ProcMap,
    pub(super) children: Box<[Node]>,
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

            f.debug_struct("")
                .field("patterns", &PatternsMarker(self.patterns))
                // .field("proc", &HandlerMarker::from(self.handler.as_ref()))
                .field("children", &self.children)
                .finish()
        }
    }
};

pub(super) struct ProcMap {
    pub(super) GET:       BoxedFPC,
    pub(super) PUT:       BoxedFPC,
    pub(super) POST:      BoxedFPC,
    pub(super) PATCH:     BoxedFPC,
    pub(super) DELETE:    BoxedFPC,
    pub(super) OPTIONS:   BoxedFPC,
    pub(super) __catch__: BoxedFPC,
} impl ProcMap {
    #[inline(always)]
    const fn lookup(&self, method: Method) -> &BoxedFPC {
        match method {
            Method::GET     => &self.GET,
            Method::PUT     => &self.PUT,
            Method::POST    => &self.POST,
            Method::PATCH   => &self.PATCH,
            Method::DELETE  => &self.DELETE,
            Method::OPTIONS => &self.OPTIONS,
            Method::HEAD    => &self.GET,
        }
    }
}

pub(super) enum Pattern {
    Static(&'static [u8]),
    Param,
} const _: () = {
    impl std::fmt::Debug for Pattern {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Param         => f.write_str(":Param"),
                Self::Static(bytes) => {
                    f.write_char('\'')?;
                    f.write_str(std::str::from_utf8(bytes).unwrap())?;
                    f.write_char('\'')?;
                    Ok(())
                },
            }
        }
    }
};


/*===== impls =====*/

impl RadixRouter {
    #[inline(always)]
    pub(crate) async fn handle(
        &self,
        req: &mut Request,
    ) -> Response {
        let res = self.0.search(req).call_bite(req).await;
        
        match req.method() {
            Method::HEAD => res.without_content(),
            _            => res,
        }
    }
}

impl Node {
    pub(super/* for test */) fn search(&self, req: &mut Request) -> &dyn FangProcCaller {
        let method = req.method();
        // SAFETY:
        // 1. `req` must be alive while `search`
        // 2. `Request` DOESN'T have method that mutates `path`,
        //    So what `path` refers to is NEVER changed by any other process
        //    while `search`
        let mut path = unsafe {req.internal_path_bytes()};

        let mut target = self;

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
                    return &*target.proc.__catch__
                }

                path = unsafe {path.get_unchecked(1..)};
                
                #[cfg(feature="DEBUG")]
                println!("[path striped prefix '/'] '{}'", path.escape_ascii());
        
                match pattern {
                    Pattern::Static(s)  => path = match path.strip_prefix(*s) {
                        Some(remaining) => remaining,
                        None            => return &*target.proc.__catch__,
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
        
                return  &*target.proc.lookup(method)
            } else {
                #[cfg(feature="DEBUG")]
                println!("not found, searching children: {:#?}", target.children);
        
                target = match target.matchable_child(path) {
                    Some(child) => child,
                    None        => return &*target.proc.__catch__,
                }
            }
        }
    }
}


/*===== utils =====*/

impl Node {
    #[inline] fn matchable_child(&self, path: &[u8]) -> Option<&Node> {
        for child in &*self.children {
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
