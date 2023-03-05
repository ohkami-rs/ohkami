#![allow(non_snake_case)]
pub(crate) mod trie_tree;

use crate::{
    fang::Fang,
    context::Context,
    handler::HandleFunc,
    request::{PathParams, Request},
};


pub(crate) struct Router<'req> {
    GET: Node<'req>,
    POST: Node<'req>,
    PATCH: Node<'req>,
    DELETE: Node<'req>,
} impl<'req> Router<'req> {
    #[inline] pub(crate) async fn handle(
        &'req self,
        mut c: Context,
        request: Request<'req>,
    ) {
        let path_params = PathParams::new();
        match request.method {
            "GET" => self.GET.handle(request.path, c, request, path_params).await,
            "POST" => self.POST.handle(request.path, c, request, path_params).await,
            "PATCH" => self.PATCH.handle(request.path, c, request, path_params).await,
            "DELETE" => self.DELETE.handle(request.path, c, request, path_params).await,
            other => c.NotImplemented::<(), _>(format!("unknown method: {other}")).send(&mut c.stream).await
        }
    }
}
struct Node<'req> {
    patterns:    &'req [(Pattern, Option</* combibed */Fang<'req>>)],
    handle_func: Option<HandleFunc<'req>>,
    children:    &'req [Node<'req>],
} impl<'req> Node<'req> {
    #[inline] fn matchable_child(&'req self, current_path: &'req str) -> Option<&'req Self> {
        for child in self.children {
            match child.patterns.first()?.0 {
                Pattern::Param  => return Some(child),
                Pattern::Str(s) => if current_path.starts_with(s) {return Some(child)}
            }
        }
        None
    }
}
enum Pattern {
    Str(&'static str),
    Param,
}


enum Handle<'req> {
    Fin,
    Continue(
        &'req Node<'req>,
        /* path */&'req str,
        Context,
        Request<'req>,
        PathParams<'req>,
    ),
}
const _: () = {
    /*
        recursion in an `async fn` requires boxing
        a recursive `async fn` must be rewritten to return a boxed `dyn Future`
        consider using the `async_recursion` crate: https://crates.io/crates/async_recursion
    */
    use Handle::*;

    impl<'req> Node<'req> {
        #[inline] async fn handle(&'req self,
            mut path: &'req str,
            mut c: Context,
            mut request: Request<'req>,
            mut path_params: PathParams<'req>,
        ) {
            let mut search_root = self;
            loop {
                (search_root, path, c, request, path_params) = match search_root._handle(path, c, request, path_params).await {
                    Fin => return,
                    Continue(_search_root, _path, _c, _request, _path_params) => (_search_root, _path, _c, _request, _path_params),
                }
            }
        }
        #[inline] async fn _handle(
            &'req self,
            mut path: &'req str,
            mut c: Context,
            mut request: Request<'req>,
            mut path_params: PathParams<'req>,
        ) -> Handle<'req> {
            for (pattern, fang) in self.patterns {
                if path.is_empty() {c.NotFound::<(), _>("").send(&mut c.stream); return Fin}
                match pattern {
                    Pattern::Str(s) => path = match path.strip_prefix(s) {
                        None => return {c.NotFound::<(), _>("").send(&mut c.stream).await; Fin},
                        Some(rem) => {
                            if let Some(fang) = fang {
                                (c, request) = fang(c, request).await;
                            }
                            rem
                        },
                    },
                    Pattern::Param => match path[1..].find('/') {
                        Some(len) => {
                            path_params.push(&path[1..1+len]);
                            path = &path[1+len..]
                        },
                        None => {
                            path_params.push(&path[1..]);
                            path = ""
                        },
                    },
                }
            }

            if path.is_empty() {
                match &self.handle_func {
                    None => {c.NotFound::<(), _>("").send(&mut c.stream).await; Fin},
                    Some(handle_func) => {handle_func(c, request, path_params).await; Fin},
                }
            } else {
                match self.matchable_child(path) {
                    None => {c.NotFound::<(), _>("").send(&mut c.stream).await; Fin},
                    Some(child) => Continue(child, path, c, request, path_params),
                }
            }
        }
    }
};
