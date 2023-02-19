use super::node::Node;

const CHILDREN_SIZE: usize = 8;

pub(super) struct Children<'router>(
    Vec<Node<'router>>
); impl<'router> Children<'router> {
    #[inline] pub(super) fn new() -> Self {
        Self(
            Vec::with_capacity(CHILDREN_SIZE)
        )
    }

    #[inline] pub(super) fn push(&mut self, node: Node) {
        self.0.push(node)
    }

    #[inline] pub(super) fn search_matches(&self, section: &str) -> Option<&Node> {
        self.0.iter().find(|node| node.pattern.matches(section))
    }
}
