use super::{Router, Tree, Procs, Node as RouterNode, Pattern as RouterPattern};
use crate::{fang::Fang, handler::Handler};


/*===== definitions =====*/
pub(crate) struct RadixTree {
    GET: Node,
    POST: Node,
    PATCH: Node,
    DELETE: Node,
}

struct Node {
    sections: &'static [Section],
    handler:  Option<Handler>,
    children: &'static [Node],
}

struct Section {
    pattern: Pattern,
    fangs:   &'static [Fang],
}
enum Pattern {
    Str(&'static str),
    Param,
    Nil,
}


/*===== mutations =====*/
impl RadixTree {
    #[inline(always)] pub(crate) fn into_router(self) -> Router {
        let (get_node, get_procs) = self.GET.into_router();
        let (post_node, post_procs) = self.POST.into_router();
        let (patch_node, patch_procs) = self.PATCH.into_router();
        let (delete_node, delete_procs) = self.DELETE.into_router();

        Router {
            tree: Tree {
                GET: get_node,
                POST: post_node,
                PATCH: patch_node,
                DELETE: delete_node,
            },
            procs: Procs {
                GET: get_procs,
                POST: post_procs,
                PATCH: patch_procs,
                DELETE: delete_procs,
            },
        }
    }
}
impl Node {
    fn into_router(self) -> (RouterNode, &'static [(&'static [Fang], Handler)]) {
        let mut procs = Vec::<(Vec<Fang>, Handler)>::new();
        let mut id = 0;
        // `self` is 0th Node
        
    }
}
