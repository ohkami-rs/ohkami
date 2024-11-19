use crate::request::Path;
use crate::{Method, Request, Response};
use crate::fang::{FangProcCaller, BoxedFPC};
use ohkami_lib::Slice;
use std::fmt::Write as _;


#[derive(Debug)]
pub(crate) struct RadixRouter {
    pub(super) GET:     Node,
    pub(super) PUT:     Node,
    pub(super) POST:    Node,
    pub(super) PATCH:   Node,
    pub(super) DELETE:  Node,
    pub(super) OPTIONS: Node,
}

pub(super) struct Node {
    pub(super) patterns:  &'static [Pattern],
    pub(super) children:  &'static [Node],
    pub(super) proc:      BoxedFPC,
    pub(super) __catch__: BoxedFPC,
} const _: () = {
    impl std::fmt::Debug for Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            struct PatternsMarker(&'static [Pattern]);
            impl std::fmt::Debug for PatternsMarker {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_list().entries(self.0).finish()
                }
            }

            f.debug_struct("")
                .field("patterns", &PatternsMarker(self.patterns))
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
        (match req.method {
            Method::GET     => &self.GET,
            Method::PUT     => &self.PUT,
            Method::POST    => &self.POST,
            Method::PATCH   => &self.PATCH,
            Method::DELETE  => &self.DELETE,
            Method::OPTIONS => &self.OPTIONS,
            Method::HEAD => {
                let mut res = self.GET.search(&mut req.path).call_bite(req).await;
                {/* not `res.drop_content()` to leave `Content-Type`, `Content-Length` */
                    res.content = crate::response::Content::None;
                }
                return res
            }
        }).search(&mut req.path).call_bite(req).await
    }
}

impl Node {
    #[inline]
    pub(super/* for test */) fn search(&self,
        path: &mut Path
    ) -> &dyn FangProcCaller {
        // SAFETY:
        // 1. `req` must be alive while `search`
        // 2. `Request` DOESN'T have method that mutates `bytes`,
        //    So what `bytes` refers to is NEVER changed by any other process
        //    while `search`
        let mut bytes = unsafe {path.normalized_bytes()};

        let mut target = self;

        #[cfg(feature="DEBUG")]
        println!("[path] '{}'", bytes.escape_ascii());

        loop {
            #[cfg(feature="DEBUG")]
            println!("[target] {:#?}", target);
            #[cfg(feature="DEBUG")]
            println!("[patterns] {:?}", target.patterns);
    
            for pattern in target.patterns {
                if bytes.is_empty() || unsafe {bytes.get_unchecked(0)} != &b'/' {
                    // At least one `pattern` to match is remaining
                    // but remaining `bytes` doesn't start with '/'
                    return &target.__catch__
                }

                bytes = unsafe {bytes.get_unchecked(1..)};
                
                #[cfg(feature="DEBUG")]
                println!("[bytes striped prefix '/'] '{}'", bytes.escape_ascii());
        
                match pattern {
                    Pattern::Static(s) => bytes = match bytes.strip_prefix(*s) {
                        Some(remaining) => remaining,
                        None            => return &target.__catch__,
                    },
                    Pattern::Param => {
                        let (param, remaining) = split_next_section(bytes);
                        unsafe {path.push_param(Slice::from_bytes(param))}
                        bytes = remaining;
                    },
                }
            }

            if bytes.is_empty() {
                #[cfg(feature="DEBUG")]
                println!("Found: {target:?}");
        
                return  &target.proc
            } else {
                #[cfg(feature="DEBUG")]
                println!("not found, searching children: {:#?}", target.children);
        
                target = match target.matchable_child(bytes) {
                    Some(child) => child,
                    None        => return &target.__catch__,
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
