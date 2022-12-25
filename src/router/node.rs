use std::str::Split;
use crate::{utils::{range::RangeList, buffer::BufRange}, result::{Result, ElseResponse}, response::Response, handler::HandleFunc, setting::MiddlewareFunc};
use super::pattern::Pattern;

pub(super) struct MiddlewareRegister {
    pub(super) just: Option<MiddlewareFunc>,
    pub(super) proccess: Vec<MiddlewareFunc>,
} impl MiddlewareRegister {
    fn new() -> Self {
        Self { just: None, proccess: Vec::new() }
    }
}

// #derive[Debug, PartialEq]
pub(super) struct Node {
    pub(super) pattern:    Pattern,
    pub(super) handler:    Option<HandleFunc>,
    pub(super) middleware: MiddlewareRegister,
    pub(super) children:   Vec<Node>,
} impl Node {
    pub fn new(pattern: Pattern) -> Self {
        Self {
            pattern,
            handler:    None,
            middleware: MiddlewareRegister::new(),
            children:   Vec::new(),
        }
    }

    pub fn search<'tree, 'req>(&'tree self,
        mut path:     Split<'req, char>,
        mut params:   RangeList,
        mut read_pos: usize,
        mut middleware_process: Vec<&'tree MiddlewareFunc>,
    ) -> Result<(
        &'tree HandleFunc,
        RangeList,
        Vec<&'tree MiddlewareFunc>,
        Option<&'tree MiddlewareFunc>,
    )> {
        if let Some(section) = path.next() {
            read_pos += 1 /* skip '/' */;
            if let Some(child) = 'search: {
                for child in &self.children {
                    if child.pattern.matches(section) {

                        tracing::debug!("search visited: {section}");
                        tracing::debug!("just: {}", if child.middleware.just.is_some() {"exists"} else {"no"});
                        tracing::debug!("proccess: {}", child.middleware.proccess.len());

                        if child.pattern.is_param() {
                            let range = BufRange::new(read_pos + 1, read_pos + section.len());
                            tracing::debug!("path param: `{}` (range: {:?})", section, range);
                            params.push(range)?;
                        }
                        for proceess in &child.middleware.proccess {

                            tracing::debug!("pushed!");

                            middleware_process.push(proceess)
                        }
                        break 'search Some(child)
                    }
                }
                None
            } {
                child.search(path, params, read_pos + section.len(), middleware_process)
            } else {
                Err(Response::NotFound(None))
            }
        } else {
            Ok((
                self.handler.as_ref()._else(|| Response::NotFound(None))?,
                params,
                middleware_process,
                self.middleware.just.as_ref()
            ))
        }
    }

    pub fn register_handler(&mut self,
        mut path: Split<'static, char>,
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
                child.register_handler(path, handler, err_msg)

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
        path:    Split<'static, char>,
        handler: HandleFunc,
    ) {
        let path = path.rev().collect::<Vec<_>>();
        self._attach(path, handler)
    }
    fn _attach(&mut self,
        mut path: Vec<&'static str>,
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

    pub(super) fn register_middleware_func(mut self,
        route:           &'static str /* already validated */,
        middleware_func: MiddlewareFunc,
        err_msg:         String,
    ) -> std::result::Result<Self, String> {
        if route.ends_with("/*") {
            let mut route = route.trim_end_matches("/*").split('/');
            { route.next(); }

            if let Some(apply_root) = self.search_apply_root(route) {

                tracing::debug!("proccess pushed!");

                apply_root.middleware.proccess.push(middleware_func)
            }

        } else {
            let mut route = route.split('/');
            { route.next(); }

            if let Some(target) = self.search_apply_root(route) {
                if target.middleware.just.is_some() {
                    return Err(err_msg)
                }

                tracing::debug!("just pushed!");

                target.middleware.just = Some(middleware_func)
            }
        }

        Ok(self)
    }
    fn search_apply_root(&mut self, mut path: Split<'static, char>) -> Option<&mut Self> {
        if let Some(section) = path.next() {
            if let Some(child) = 'search: {
                for child in &mut self.children {
                    if child.pattern.matches(section) {

                        tracing::debug!("search_apply_root visited: {section}");

                        break 'search Some(child)
                    }
                }
                None
            } {
                child.search_apply_root(path)
            } else {
                tracing::debug!("search_apply_root returned None");
                None
            }
        } else {
            tracing::debug!("search_apply_root returned `{:?}`", self.pattern);
            Some(self)
        }
    }
}