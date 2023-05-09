mod trie_tree;
mod radix_tree;

use async_std::net::TcpStream;
use crate::{Fang, handler::Handler, Context, Request, request::{Method, PathParams}};


/*===== definitions =====*/
pub(crate) struct Router {
    tree:  Tree,
    procs: Procs,
}

struct Tree {
    GET:    Node,
    POST:   Node,
    PATCH:  Node,
    DELETE: Node,
}
struct Node {
    id: Option<usize>,
    patterns: &'static [Pattern],
    children: &'static [Node],
}
enum Pattern {
    Str(&'static str),
    Param,
}

struct Procs {
    GET:    &'static [(&'static [Fang], Handler)],
    POST:   &'static [(&'static [Fang], Handler)],
    PATCH:  &'static [(&'static [Fang], Handler)],
    DELETE: &'static [(&'static [Fang], Handler)],
}


/*===== impls =====*/
impl Router {
    pub(crate) async fn handle(
        &self,
        mut c: Context,
        mut stream: TcpStream,
        mut request: Request,
    ) {
        let Some((id, path_params)) = self.search(request.method(), request.path()) else {
            c.NotFound::<(), _>("").send(&mut stream);
            return;
        };
        let (fangs, handler) = unsafe {self.procs.get(request.method(), id)};

        for fang in fangs {
            (c, request) = fang(c, request).await
        }
        handler(stream, c, request, path_params).await
    }

    #[inline(always)] fn search(&self, method: &Method, path: &str) -> Option<(/* id */usize, PathParams)> {
        let path_params = PathParams::new();
        match method {
            Method::GET => self.tree.GET.search(path, path_params),
            Method::POST => self.tree.POST.search(path, path_params),
            Method::PATCH => self.tree.PATCH.search(path, path_params),
            Method::DELETE => self.tree.DELETE.search(path, path_params),
        }
    }
}

impl Procs {
    /// without checking if `method` and `id` is valid
    #[inline(always)] unsafe fn get(
        &self,
        method: &Method,
        id: usize,
    ) -> (&[Fang], Handler) {
        match method {
            Method::GET => self.GET[id],
            Method::POST => self.POST[id],
            Method::PATCH => self.PATCH[id],
            Method::DELETE => self.DELETE[id],
        }
    } 
}

impl Node {
    fn search(&self, mut path: &str, mut path_params: PathParams) -> Option<(/* id */usize, PathParams)> {
        let path_len = path.len();
        let mut param_section_start = 1/* skip initial '/' */;

        let mut search_target = self;
        loop {
            for pattern in search_target.patterns {
                path = path.strip_prefix('/')?;
                match pattern {
                    Pattern::Str(s) => {
                        path = path.strip_prefix(s)?;
                        param_section_start += s.len() + 1/* '/' */;
                    }
                    Pattern::Param  => {
                        match path.find('/') {
                            Some(rem_len) => {
                                path = &path[rem_len+1..];
                                path_params.push(param_section_start..(param_section_start+rem_len));
                                param_section_start += rem_len + 1/* skip '/' */;
                            }
                            None => {
                                path = "";
                                path_params.push(param_section_start..path_len);
                            }
                        }
                    }
                }
            }

            if path.is_empty() {
                return Some((search_target.id?, path_params))
            } else {
                search_target = search_target.matchable_child(path)?
            }
        }
    }

    #[inline(always)] fn matchable_child(&self, path: &str) -> Option<&Self> {
        for child in self.children {
            if child.patterns.first()?.is_matchable_to(path) {
                return Some(child)
            }
        }
        None
    }
}

impl Pattern {
    #[inline(always)] fn is_matchable_to(&self, path: &str) -> bool {
        match self {
            Self::Param  => true,
            Self::Str(s) => match path.find('/') {
                Some(slash) => &path[..slash] == *s,
                None        => path == *s,
            },
        }
    }
}
