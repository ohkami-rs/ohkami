use std::{str::Split, rc::Rc};
use crate::{utils::{range::RangeList, buffer::BufRange}, result::{Result, ElseResponse}, response::Response, handler::HandleFunc, setting::MiddlewareFunc};
use super::pattern::Pattern;

// #derive[Debug, PartialEq]
pub(super) struct Node {
    pub(super) pattern:     Pattern,
    pub(super) handler:     Option<HandleFunc>,
    pub(super) middlewares: Vec<MiddlewareFunc>,
    pub(super) children:    Vec<Node>,
} impl Node {
    pub fn new(pattern: Pattern) -> Self {
        Self {
            pattern,
            handler:     None,
            middlewares: Vec::new(),
            children:    Vec::new(),
        }
    }

    pub fn search_handler<'req>(&self,
        mut path:     Split<'req, char>,
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
                child.search_handler(path, params, read_pos + section.len())
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

    pub(super) fn register_middleware_func(&mut self,
        path:            &'static str /* already validated */,
        middleware_func: MiddlewareFunc,
    ) {
        if path.ends_with('*') {
            let apply_root = self.search_apply_root(
                path.trim_end_matches('*')[1..].split('/')
            );

            // ====================
            // TODO
            // - clone はできないが、apply root に置いておいて search 時に通ったものを順次実行すれば良さそう
            // - 問題は「他の route の途中部分として現れうる route 」を 〜/* ではなく just route として指定された場合.
            //   これについては「通ったら実行される」と「そこでストップした場合のみ実行される」を区別するしかないか？
            //   その場合、１node が持てる just route middleware func は１つなので、handler と同様 register で Result を返す
            // ====================

        } else {
            if let Some(target) = self.search_apply_root(
                path[1..].split('/')
            ) {
                target.middlewares.push(middleware_func)
            }
        }
    }
    fn search_apply_root(&mut self, mut path: Split<'static, char>) -> Option<&mut Self> {
        if let Some(section) = path.next() {
            if let Some(child) = 'search: {
                for child in &mut self.children {
                    if child.pattern.matches(section) {
                        break 'search Some(child)
                    }
                }
                None
            } {
                child.search_apply_root(path)
            } else {
                None
            }
        } else {Some(self)}
    }
}