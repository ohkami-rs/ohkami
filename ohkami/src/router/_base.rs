use super::_util::ID;
use crate::fang::{BoxedFPC, Fangs, Handler};
use std::{sync::Arc, ops::Range, collections::HashSet};


pub(crate) struct Router {
    id:      ID,
    routes:  HashSet<&'static str>,
    GET:     Node,
    PUT:     Node,
    POST:    Node,
    PATCH:   Node,
    DELETE:  Node,
    OPTIONS: Node,
}

struct Node {
    pattern:  Option<Pattern>,
    handler:  Option<Handler>,
    fangses:  FangsList,
    children: Vec<Node>
}

enum Pattern {
    Static { route: &'static str, range: Range<usize> },
    Param
}

#[derive(Clone)]
pub(super) struct FangsList(Vec<(
    ID,
    Arc<dyn Fangs>
)>);
impl FangsList {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add(&mut self, id: ID, fangs: Arc<dyn Fangs>) {
        if self.0.iter().find(|(_id, _)| *_id == id).is_none() {
            self.0.push((id, fangs));
        }
    }
    fn extend(&mut self, another: Self) {
        for (id, fangs) in another.0.into_iter() {
            self.add(id, fangs)
        }
    }

    fn into_proc_with(self, handler: Handler) -> BoxedFPC {
        let mut iter = self.into_iter();

        match iter.next() {
            None => handler.into(),
            Some(most_inner) => iter.fold(
                most_inner.build(handler.into()),
                |proc, fangs| fangs.build(proc)
            )
        }
    }

    /// yield from most inner fangs
    fn iter(&self) -> impl Iterator<Item = &Arc<dyn Fangs>> {
        self.0.iter()
            .map(|(_, fangs)| fangs)
    }
    /// yield from most inner fangs
    fn into_iter(self) -> impl Iterator<Item = Arc<dyn Fangs>> {
        self.0.into_iter()
            .map(|(_, fangs)| fangs)
    }
}
