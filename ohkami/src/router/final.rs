use super::{util, base};
use crate::fang::{FangProcCaller, BoxedFPC, handler::Handler};
use crate::{request::Path, response::Content};
use crate::{Method, Request, Response};
use ohkami_lib::Slice;


#[allow(non_snake_case)]
pub(crate) struct Router {
    GET:     Node,
    PUT:     Node,
    POST:    Node,
    PATCH:   Node,
    DELETE:  Node,
    OPTIONS: Node,
}

pub(super) struct Node {
    pattern:  Pattern,
    proc:     BoxedFPC,
    catch:    BoxedFPC,
    children: &'static [Node],

    #[cfg(feature="openapi")]
    openapi_operation: Option<crate::openapi::Operation>
}

#[derive(PartialEq)]
enum Pattern {
    Static(&'static [u8]),
    Param
}


impl Router {
    pub(crate) async fn handle(&self, req: &mut Request) -> Response {
        let mut res = 'handle: {
            (match req.method {
                Method::GET     => &self.GET,
                Method::PUT     => &self.PUT,
                Method::POST    => &self.POST,
                Method::PATCH   => &self.PATCH,
                Method::DELETE  => &self.DELETE,
                Method::OPTIONS => &self.OPTIONS,

                Method::HEAD => {
                    let mut res = self.GET.search(&mut req.path).call_bite(req).await;

                    /* not `res.drop_content()` to keep `Content-Type`, `Content-Length` */
                    res.content = Content::None;

                    break 'handle res
                }

            }).search(&mut req.path).call_bite(req).await
        };

        res.complete();

        res
    }

    #[cfg(feature="openapi")]
    pub(crate) fn gen_openapi_doc<'r>(
        &self,
        routes: impl Iterator<Item = (&'r str, impl Iterator<Item = Method>)>,
        metadata: crate::openapi::OpenAPI,
    ) -> crate::openapi::document::Document {
        let mut doc = crate::openapi::document::Document::new(
            metadata.title,
            metadata.version,
            metadata.servers
        );

        for (route, methods) in routes {
            crate::DEBUG!("[gen_openapi_doc] route = `{route}`");

            assert!(route.starts_with('/'));

            let (openapi_path, openapi_path_param_names) = {
                let (mut path, mut params) = (String::new(), Vec::new());
                for segment in route.split('/').skip(1/* head empty */) {
                    path += "/";
                    if let Some(param) = segment.strip_prefix(':') {
                        path += &["{", param, "}"].concat();
                        params.push(param);
                    } else {
                        path += segment;
                    }
                }
                (path, params)
            };

            let mut operations = crate::openapi::paths::Operations::new();
            for method in methods {
                let (openapi_method, router) = match method {
                    Method::GET    => ("get",    &self.GET),
                    Method::PUT    => ("put",    &self.PUT),
                    Method::POST   => ("post",   &self.POST),
                    Method::PATCH  => ("patch",  &self.PATCH),
                    Method::DELETE => ("delete", &self.DELETE),
                    _ => continue
                };
                
                let mut path = unsafe {crate::request::Path::from_str_unchecked(
                    // this is intended even when route == "/", then to "",
                    // samely as `Path::init_with_request_bytes`
                    route.trim_end_matches('/')
                )};

                crate::DEBUG!("[gen_openapi_doc] searching `{openapi_method} {route}`");

                let (target, true) = router.search_target(&mut path) else {
                    continue
                };
                let Some(mut operation) = target.openapi_operation.clone() else {
                    continue
                };

                crate::DEBUG!("[gen_openapi_doc] found");
                        
                for param_name in &openapi_path_param_names {
                    operation.assign_path_param_name(param_name.to_string());
                }
                for security_scheme in operation.iter_securitySchemes() {
                    doc.register_securityScheme_component(security_scheme);
                }
                for schema_component in operation.refize_schemas() {
                    doc.register_schema_component(schema_component);
                }
                operations.register(openapi_method, operation);
            }

            doc = doc.path(openapi_path, operations);
        }

        doc
    }
}

impl Node {
    /// ## Precondition
    /// 
    /// `patterns`s of all `Node`s belonging to this tree MUST be:
    /// 
    /// 1. all `Pattern::Static`s are sorted in reversed alphabetical order
    /// 2. zero or one `Pattern::Param` exists at the end
    #[inline(always)]
    fn search(&self, path: &mut Path) -> &dyn FangProcCaller {
        let (target, hit) = self.search_target(path);
        if hit {&target.proc} else {&target.catch}
    }

    pub(super) fn search_target(&self, path: &mut Path) -> (&Self, bool) {
        let mut bytes = unsafe {path.normalized_bytes()};

        /*
            When `GET / HTTP/1.1` is coming, here
            
            * `bytes` is `b""` (by the normalization).
            * `self.pattern` is, an any pattern if compressed single-child,
              or `Static(b"")` by default.

            If compressed, the router has handlers only at or under the single pattern:

            ```
            /abc
            ├── /xyz  # /abc/xyz
            ├── /def  # /abc/def
            :
            ```

            and of course `self.pattern.take_through` returns `None`,
            `search` returns the catcher because no handler is registered to `/`.

            If not compressed, in other words, the router has multiple canidates for
            handler-routes under `/`:

            ```
            .             # /
            ├── /xyz      # /xyz
            │   ├── /pqr  # /xyz/pqr
            │   └── /def  # /xyz/def
            ├── /abc      # /abc
            :
            ```

            and `self.pattern.take_through` successes by matching `b""`,
            then `search` returns the proc (the handler if user registered, or
            NotFound handler if not, with fangs).
            When `GET /abc HTTP/1.1` is coming, this `self.pattern.take_through`
            successes with `Some(b"/abc")`, then we just perform `bytes = b"/abc"`
            and go to `'next_target` loop.
        */
        
        if let Some(remaining) = self.pattern.take_through(bytes, path) {
            if remaining.is_empty() {
                return (&self, true)
            } else {
                bytes = remaining
            }
        } else {
            return (&self, false)
        }

        let mut target = self;
        'next_target: loop {
            for child in target.children {
                if let Some(remaining) = child.pattern.take_through(bytes, path) {
                    if remaining.is_empty() {
                        return (&child, true)
                    } else {
                        bytes  = remaining;
                        target = child;
                        continue 'next_target
                    }
                }
            }; return (&target, false)
        }
    }
}

impl Pattern {
    /// ## Precondition
    /// 
    /// `self`, if `Static`, must hold bytes starting with `/` e.g. `/abc`, `/`, `/abc/xyz`
    #[inline]
    fn take_through<'b>(
        &self,
        bytes: &'b [u8],
        path:  &mut Path
    ) -> Option<&'b [u8]/* remaining part of `bytes` */> {
        match self {
            Pattern::Static(s) => {
                let size = s.len();
                if bytes.len() >= size && *s == unsafe {bytes.get_unchecked(..size)} {
                    Some(unsafe {bytes.get_unchecked(size..)})
                } else {
                    None
                }
            }
            Pattern::Param => {
                if bytes.len() >= 2
                && *unsafe {bytes.get_unchecked(0)} == b'/'
                && *unsafe {bytes.get_unchecked(1)} != b'/' {
                    let (param, remaining) = util::split_next_section(unsafe {bytes.get_unchecked(1..)});
                    unsafe {path.push_param(Slice::from_bytes(param))};
                    Some(remaining)
                } else {
                    None
                }
            }
        }
    }
}


const _: (/* conversions */) = {
    impl From<base::Router> for Router {
        fn from(base: base::Router) -> Self {
            Router {
                GET:     Node::from(base.GET),
                PUT:     Node::from(base.PUT),
                POST:    Node::from(base.POST),
                PATCH:   Node::from(base.PATCH),
                DELETE:  Node::from(base.DELETE),
                OPTIONS: Node::from(base.OPTIONS),
            }
        }
    }
    
    impl From<base::Node> for Node {
        fn from(mut base: base::Node) -> Self {
            /* skip compression on edge runtimes */
            #[cfg(feature="__rt_native__")]
            /* compress: merge single-child static pattern and compress routing tree */
            while base.children.len() == 1
               && base.handler.is_none()
               && base.pattern.as_ref().is_none_or(|p| p.is_static())
               && base.children[0].pattern.as_ref().unwrap(/* not root */).is_static()
            {
                let child = base.children.pop().unwrap(/* base.children.len() == 1 */);
                base.children = child.children;
                base.handler = child.handler;
                base.fangses.append(child.fangses);
                base.pattern = Some(match base.pattern {
                    None    => child.pattern.unwrap(/* not root */),
                    Some(p) => p.merge_statics(child.pattern.unwrap(/* not root */)).unwrap(/* both are Pattern::Static */)
                });
            }

            base.children.sort_by(|a, b| match (
                a.pattern.as_ref().unwrap(/* not root */),
                b.pattern.as_ref().unwrap(/* not root */)
            ) {
                (base::Pattern::Static(a), base::Pattern::Static(b)) => a.cmp(b).reverse(),
                (base::Pattern::Static(_), base::Pattern::Param (_)) => std::cmp::Ordering::Less,
                (base::Pattern::Param (_), base::Pattern::Static(_)) => std::cmp::Ordering::Greater,
                _                                                    => std::cmp::Ordering::Equal
            });

            #[cfg(feature="openapi")] let has_handler = base.handler.is_some();

            let proc = base.fangses.clone().into_proc_with(base.handler.unwrap_or(Handler::default_not_found()));
            #[cfg(feature="openapi")] let (proc, openapi_operation) = (proc.0, has_handler.then_some(proc.1));

            let catch = base.fangses.into_proc_with(Handler::default_not_found());
            #[cfg(feature="openapi")] let catch = catch.0;

            Node {
                pattern:  base.pattern.map(Pattern::from).unwrap_or(Pattern::Static(b"")),
                children: base.children.into_iter().map(Node::from).collect::<Vec<_>>().leak(),

                proc,
                catch,

                #[cfg(feature="openapi")]
                openapi_operation
            }
        }
    }

    impl From<base::Pattern> for Pattern {
        fn from(base: base::Pattern) -> Self {
            match base {
                base::Pattern::Param(_)  => Self::Param,
                base::Pattern::Static(s) => Self::Static(match s {
                    std::borrow::Cow::Borrowed(s) => s.as_bytes(),
                    std::borrow::Cow::Owned(s) => s.leak().as_bytes(),
                }),
            }
        }
    }
};

#[cfg(feature="DEBUG")]
const _: (/* Debugs */) = {
    impl std::fmt::Debug for Router {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("FinalRouter")
                .field("GET", &self.GET)
                .field("PUT", &self.PUT)
                .field("POST", &self.POST)
                .field("PATCH", &self.PATCH)
                .field("DELETE", &self.DELETE)
                .field("OPTIONS", &self.OPTIONS)
                .finish()
        }
    }

    impl std::fmt::Debug for Node {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut d = f.debug_struct("");
            d
                .field("pattern",  &self.pattern)
                .field("children", &self.children);

            #[cfg(feature="openapi")] {
                struct DebugOperaion<'d>(Option<&'d crate::openapi::Operation>);
                impl std::fmt::Debug for DebugOperaion<'_> {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str(if self.0.is_some() {
                            "Some({operation})"
                        } else {
                            "None"
                        })
                    }
                }
                d.field("operation", &DebugOperaion(self.openapi_operation.as_ref()));
            }
            
            d.finish()
        }
    }

    impl std::fmt::Debug for Pattern {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                Self::Param     => ":param",
                Self::Static(s) => std::str::from_utf8(s).unwrap(),
            })
        }
    }
};
