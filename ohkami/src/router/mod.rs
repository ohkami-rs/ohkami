#![allow(non_snake_case)]
pub(crate) mod trie_tree;

use async_std::{sync::Arc, net::TcpStream};
use crate::{
    fang::Fang,
    context::Context,
    handler::Handler,
    request::{PathParams, Request},
};


pub(crate) struct Router<'req> {
    GET: Node<'req>,
    POST: Node<'req>,
    PATCH: Node<'req>,
    DELETE: Node<'req>,
} impl<'req, 'buf: 'req> Router<'req> {
    #[inline] pub(crate) async fn handle(
        &'req self,
        c: Context,
        mut stream: TcpStream,
        request: Request<'buf>,
    ) {
        let path_params = PathParams::new();
        let result = match request.method {
            "GET" => self.GET.handle(request.path, c, stream, request, path_params).await,
            "POST" => self.POST.handle(request.path, c, stream, request, path_params).await,
            "PATCH" => self.PATCH.handle(request.path, c, stream, request, path_params).await,
            "DELETE" => self.DELETE.handle(request.path, c, stream, request, path_params).await,
            other => c.NotImplemented::<(), _>(format!("unknown method: {other}")).send(&mut stream).await
        };
        result
    }
}
struct Node<'req> {
    patterns:    &'req [(Pattern, Option</* combibed */Fang<'req>>)],
    handler:     Option<Handler<'req>>,
    children:    &'req [Node<'req>],
} impl<'req> Node<'req> {
    #[inline] fn matchable_child(&'req self, current_path: &'req str) -> Option<&'req Self> {
        for child in self.children {
            match child.patterns.first()?.0 {
                Pattern::Nil    => unreachable!(),
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
    Nil,
}


const _: () = {
    /*
        recursion in an `async fn` requires boxing
        a recursive `async fn` must be rewritten to return a boxed `dyn Future`
        consider using the `async_recursion` crate: https://crates.io/crates/async_recursion
    */

    impl<'req, 'buf: 'req> Node<'req> {
        #[inline] async fn handle(&'req self,
            mut path: &'req str,
            mut c: Context,
            mut stream: TcpStream,
            mut request: Request<'buf>,
            mut path_params: PathParams<'req>,
        ) {
            let mut search_root = self;

            loop {
                for (pattern, fang) in search_root.patterns {
                    if path.is_empty() {return c.NotFound::<(), _>("").send(&mut stream).await}
                    match pattern {
                        Pattern::Str(s) => path = match path.strip_prefix(s) {
                            None => return c.NotFound::<(), _>("").send(&mut stream).await,
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
                    match &self.handler {
                        None => return c.NotFound::<(), _>("").send(&mut stream).await,
                        Some(handler) => return handler(stream, c, request, path_params).await,
                    }
                } else {
                    match self.matchable_child(path) {
                        None => return c.NotFound::<(), _>("").send(&mut stream).await,
                        Some(child) => search_root = child,
                    }
                }
            }
        }
    }
};
