pub mod route; 
pub(crate) mod trie_tree;

use crate::{
    fang::Fang,
    context::Context,
    handler::HandleFunc,
    request::{PathParams, Request},
};

pub(crate) struct Router<'router> {
    GET: Node<'router>,
    POST: Node<'router>,
    PATCH: Node<'router>,
    DELETE: Node<'router>,
} impl<'req, 'router: 'req> Router<'router> {
    #[inline] pub(crate) fn search(
        &'req self,
        c: Context,
        request: Request<'req>,
    ) -> (
        Context,
        Request<'req>,
        Option<(
            &'req HandleFunc<'req>,
            PathParams<'req>,
        )>
    ) {
        let path_params = PathParams::new();
        match request.method {
            "GET" => self.GET.search(request.path, c, request, path_params),
            "POST" => self.POST.search(request.path, c, request, path_params),
            "PATCH" => self.PATCH.search(request.path, c, request, path_params),
            "DELETE" => self.DELETE.search(request.path, c, request, path_params),
            _ => return (c, request, None)
        }
    }
}
struct Node<'router> {
    patterns:    &'static [Pattern],
    fangs:       &'router [Fang<'router>],
    handle_func: Option<HandleFunc<'router>>,
    children:    &'router [Node<'router>],
}
enum Pattern {
    Str(&'static str),
    Param,
}

const _: () = {
    impl<'req, 'router: 'req> Node<'router> {
        #[inline] fn search(
            &self,
            mut path: &'req str,
            c: Context,
            request: Request<'req>,
            mut path_params: PathParams,
        ) -> (
            Context,
            Request,
            Option<(
                &'req HandleFunc<'req>,
                PathParams<'req>,
            )>
        ) {

        }
    }
};
