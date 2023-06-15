use crate::{Request, Context};

/*===== defs =====*/
pub(crate) struct RadixRouter {
    GET: Node,
    PUT: Node,
    POST: Node,
    HEAD: Node,
    PATCH: Node,
    DELETE: Node,
    OPTIONS: Node,
}

struct Node {

}


/*===== impls =====*/
impl RadixRouter {
    pub(crate) async fn handle(&self, c: Context, req: Request) {
        todo!()
    }
}
