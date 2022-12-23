use std::str::Split;
use crate::{utils::{range::RangeList, buffer::BufRange}, result::{Result, ElseResponse}, response::Response, handler::HandleFunc};
use super::pattern::Pattern;

// #derive[Debug, PartialEq]
pub(super) struct Node<'p> {
    pub(super) pattern:  Pattern<'p>,
    pub(super) handler:  Option<HandleFunc>,
    pub(super) children: Vec<Node<'p>>,
} impl<'p> Node<'p> {
    pub fn new(pattern: Pattern<'p>) -> Self {
        Self {
            pattern,
            handler:  None,
            children: Vec::new(),
        }
    }

    pub fn search(&self,
        mut path:     Split<'p, char>,
        mut params:   RangeList,
        mut read_pos: usize,
    ) -> Result<(&HandleFunc, RangeList)> {
        if let Some(section) = path.next() {
            read_pos += 1 /* skip '/' */;
            if let Some(child) = 'search: {
                for child in &self.children {
                    if child.pattern.matches(section) {
                        if child.pattern.is_param() {
                            let range = BufRange::new(read_pos + 1, read_pos + section.len());
                            tracing::debug!("path param: `{}` (range: {:?})", section, range);
                            params.push(range)?;
                        }
                        break 'search Some(child)
                    }
                }
                None
            } {
                child.search(path, params, read_pos + section.len())
            } else {
                Err(Response::NotFound(None))
            }
        } else {
            Ok((
                self.handler.as_ref()._else(|| Response::NotFound(None))?,
                params
            ))
        }
    }

    pub fn register(&mut self,
        mut path: Split<'p, char>,
        handler:  HandleFunc,
        err_msg:  String,
    ) -> std::result::Result<(), String> {
        if let Some(section) = path.next() {
            let pattern = Pattern::from(section);
            if let Some(child) = 'search: {
                for child in &mut self.children {
                    if child.pattern.is(&pattern) {
                        break 'search Some(child)
                    }
                }
                None
            } {
                child.register(path, handler, err_msg)

            } else {
                let mut new_branch = Node::new(pattern);
                new_branch.attach(path, handler);
                self.children.push(new_branch);
                Ok(())
            }

        } else {
            Err(err_msg)
        }
    }

    fn attach(&mut self,
        path:    Split<'p, char>,
        handler: HandleFunc,
    ) {
        let path = path.rev().collect::<Vec<_>>();
        self._attach(path, handler)
    }
    fn _attach(&mut self,
        mut path: Vec<&'p str>,
        handler:  HandleFunc,
    ) {
        if let Some(section) = path.pop() {
            let mut new_node = Node::new(Pattern::from(section));
            new_node._attach(path, handler);
            self.children.push(new_node)
        } else {
            self.handler = Some(handler)
        }
    }
}