mod trie_tree;
// mod radix_tree;

use async_std::net::TcpStream;
use crate::{Fang, handler::Handler, Context, Request, request::{Method, PathParams}};


/*===== definitions =====*/
pub(crate) struct Router {
    GET:    Node,
    POST:   Node,
    PATCH:  Node,
    DELETE: Node,
}
struct Node {
    handler:  Option<Handler>,
    patterns: &'static [Pattern],
    fangs:    &'static [Fang],
    children: &'static [Node],
}
enum Pattern {
    Str(&'static str),
    Param,
}


/*===== impls =====*/
impl Router {
    pub(crate) async fn handle(
        &self,
        mut c: Context,
        mut stream: TcpStream,
        mut request: Request,
    ) {
        let Some((target, params)) = self.search(request.method(), request.path()) else {
            c.NotFound::<(), _>("").send(&mut stream);
            return;
        };

        for fang in target.fangs {
            (c, request) = fang(c, request).await
        }
        let handler = unsafe {(&target.handler).as_ref().unwrap_unchecked()} ;
        handler(stream, c, request, params).await
    }

    #[inline(always)] fn search(&self, method: &Method, path: &str) -> Option<(&Node, PathParams)> {
        let path_params = PathParams::new();
        match method {
            Method::GET => self.GET.search(path, path_params),
            Method::POST => self.POST.search(path, path_params),
            Method::PATCH => self.PATCH.search(path, path_params),
            Method::DELETE => self.DELETE.search(path, path_params),
        }
    }
}

impl Node {
    fn search(&self, mut path: &str, mut path_params: PathParams) -> Option<(&Self, PathParams)> {
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

            if path.is_empty()  {
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
