use std::str::Split;
use crate::{utils::map::StrMap, result::{Result, ElseResponse}, response::Response};

use super::{pattern::Pattern, Handler};

#[derive(PartialEq, Debug)]
pub(super) struct Node<'p> {
    pub(super) pattern:  Pattern<'p>,
    pub(super) handler:  Option<Handler>,
    pub(super) children: Vec<Node<'p>>,
} impl<'p> Node<'p> {
    pub fn new(section: &'p str) -> Self {
        Self {
            pattern:  Pattern::from(section),
            handler:  None,
            children: Vec::new(),
        }
    }

    pub fn search(&self,
        mut path: Split<'p, char>,
        mut params: StrMap
    ) -> Result<(&Handler, StrMap)> {
        if let Some(section) = path.next() {
            if let Some(child) = 'search: {
                for child in &self.children {
                    let (is_match, param) = child.pattern.matches(section);
                    if is_match {
                        break 'search Some(child)
                    }
                }
                None
            } {
                child.search(path)
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
        handler:  Handler,
        err_msg:  String,
    ) -> std::result::Result<(), String> {
        if let Some(section) = path.next() {
            if let Some(child) = 'search: {
                for child in &mut self.children {
                    if child.pattern.matches(section) {
                        break 'search Some(child)
                    }
                }
                None
            } {
                child.register(path, handler, err_msg)

            } else {
                let mut new_branch = Node::new(section);
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
        handler: Handler,
    ) {
        let path = path.rev().collect::<Vec<_>>();
        self._attach(path, handler)
    }
    fn _attach(&mut self,
        mut path: Vec<&'p str>,
        handler:  Handler,
    ) {
        if let Some(section) = path.pop() {
            let mut new_node = Node::new(section);
            new_node._attach(path, handler);
            self.children.push(new_node)
        } else {
            self.handler = Some(handler)
        }
    }
}