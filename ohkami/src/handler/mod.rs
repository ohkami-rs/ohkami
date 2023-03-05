#![allow(non_snake_case)]
pub mod into_handlefunc;

use std::{pin::Pin, future::Future};
use crate::{
    context::Context,
    router::route::HandleRoute,
    request::{Request, PathParams},
};

pub struct Handler<'router> {
    pub(crate) route:  HandleRoute,
    pub(crate) GET:    Option<HandleFunc<'router>>,
    pub(crate) POST:   Option<HandleFunc<'router>>,
    pub(crate) PATCH:  Option<HandleFunc<'router>>,
    pub(crate) DELETE: Option<HandleFunc<'router>>,
}

pub(crate) type HandleFunc<'router> =
    Box<dyn
        Fn(Context, Request<'router>, PathParams<'router>) -> Pin<
            Box<dyn
                Future<Output = ()>
                + Send
            >
        > + Send + Sync
    >
;

impl<'router> Handler<'router> {
    pub fn GET(mut self, handle_func: HandleFunc<'router>) -> Self {
        self.GET.replace(handle_func);
        self
    }
    pub fn POST(mut self, handle_func: HandleFunc<'router>) -> Self {
        self.POST.replace(handle_func);
        self
    }
    pub fn PATCH(mut self, handle_func: HandleFunc<'router>) -> Self {
        self.PATCH.replace(handle_func);
        self
    }
    pub fn DELETE(mut self, handle_func: HandleFunc<'router>) -> Self {
        self.DELETE.replace(handle_func);
        self
    }
}
