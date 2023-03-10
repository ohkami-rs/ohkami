#![allow(non_snake_case)]
pub mod into_handler;
pub mod route;

use std::{pin::Pin, future::Future};
use crate::{
    context::Context,
    request::{Request, PathParams},
};
use async_std::net::TcpStream;
use route::HandlerRoute;

pub struct Handlers {
    pub(crate) route:  HandlerRoute,
    pub(crate) GET:    Option<Handler>,
    pub(crate) POST:   Option<Handler>,
    pub(crate) PATCH:  Option<Handler>,
    pub(crate) DELETE: Option<Handler>,
}
pub(crate) type Handler =
    Box<dyn
        Fn(TcpStream, Context, Request, PathParams) -> Pin<
            Box<dyn
                Future<Output = ()>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
;

impl Handlers {
    pub fn GET(mut self, handle_func: Handler) -> Self {
        self.GET.replace(handle_func);
        self
    }
    pub fn POST(mut self, handle_func: Handler) -> Self {
        self.POST.replace(handle_func);
        self
    }
    pub fn PATCH(mut self, handle_func: Handler) -> Self {
        self.PATCH.replace(handle_func);
        self
    }
    pub fn DELETE(mut self, handle_func: Handler) -> Self {
        self.DELETE.replace(handle_func);
        self
    }
}
