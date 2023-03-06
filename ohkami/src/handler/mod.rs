#![allow(non_snake_case)]
pub mod into_handler;
pub mod route;

use std::{pin::Pin, future::Future};
use crate::{
    context::Context,
    request::{Request, PathParams},
};
use route::HandleRoute;

pub struct Handlers<'router> {
    pub(crate) route:  HandleRoute,
    pub(crate) GET:    Option<Handler<'router>>,
    pub(crate) POST:   Option<Handler<'router>>,
    pub(crate) PATCH:  Option<Handler<'router>>,
    pub(crate) DELETE: Option<Handler<'router>>,
}
pub(crate) type Handler<'router> =
    Box<dyn
        Fn(Context, Request<'router>, PathParams<'router>) -> Pin<
            Box<dyn
                Future<Output = ()>
                + Send
            >
        > + Send + Sync
    >
;

impl<'router> Handlers<'router> {
    pub fn GET(mut self, handle_func: Handler<'router>) -> Self {
        self.GET.replace(handle_func);
        self
    }
    pub fn POST(mut self, handle_func: Handler<'router>) -> Self {
        self.POST.replace(handle_func);
        self
    }
    pub fn PATCH(mut self, handle_func: Handler<'router>) -> Self {
        self.PATCH.replace(handle_func);
        self
    }
    pub fn DELETE(mut self, handle_func: Handler<'router>) -> Self {
        self.DELETE.replace(handle_func);
        self
    }
}
