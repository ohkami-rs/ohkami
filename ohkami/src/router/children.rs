use crate::error::Error;
use super::node::Node;

const ARRAY_SIZE: usize = 8;

pub(super) struct Children<'router> {
    array:    [Option<Node<'router>>; ARRAY_SIZE],
    next_pos: usize,
}
impl<'router> Children<'router> {
    pub(super) fn new() -> Self {
        Self {
            array: [
                None, None, None, None,
                None, None, None, None,
            ],
            next_pos: 0,
        }
    }

    pub(super) fn push(&mut self, node: Node) -> crate::Result<()> {
        if self.next_pos == ARRAY_SIZE {
            return Err(Error::in_const_value("ARRAY_SIZE"))
        }

        self.array[self.next_pos].replace(node);
        self.next_pos += 1;

        Ok(())
    }

    pub(super) fn search_matches(&self, request_path_section: &str) -> Option<&Node> {
        for n in &self.array[0..self.next_pos] {
            if n.is_some_and(|node| node.pattern.matches(request_path_section)) {
                return &n
            }
        }
        None
    }
}
