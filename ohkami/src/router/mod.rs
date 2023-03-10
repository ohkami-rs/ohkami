#![allow(non_snake_case)]
pub(crate) mod trie_tree;

use async_std::net::TcpStream;
use crate::{
    fang::Fang,
    context::Context,
    handler::Handler,
    request::{PathParams, Request, Method},
};


pub(crate) struct Router {
    GET: Node,
    POST: Node,
    PATCH: Node,
    DELETE: Node,
} impl Router {
    #[inline] pub(crate) async fn handle(
        &self,
        c: Context,
        mut stream: TcpStream,
        request: Request,
    ) {
        let path_params = PathParams::new();
        let result = match request.method {
            Method::GET => self.GET.handle(&(request.path().to_owned()), c, stream, request, path_params).await,
            Method::POST => self.POST.handle(&(request.path().to_owned()), c, stream, request, path_params).await,
            Method::PATCH => self.PATCH.handle(&(request.path().to_owned()), c, stream, request, path_params).await,
            Method::DELETE => self.DELETE.handle(&(request.path().to_owned()), c, stream, request, path_params).await,
        };
        result
    }
}
struct Node {
    patterns: &'static [(Pattern, Option</* combibed */Fang>)],
    handler:  Option</* AfterFangs-combined */Handler>,
    children: &'static [Node],
} impl Node {
    #[inline] fn matchable_child(&self, current_path: &str) -> Option<&Self> {
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

    impl Node {
        #[inline] async fn handle(&self,
            mut path: &str,
            mut c: Context,
            mut stream: TcpStream,
            mut request: Request,
            mut path_params: PathParams,
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
