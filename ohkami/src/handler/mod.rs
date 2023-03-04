pub mod into_handlefunc;

use std::{pin::Pin, future::Future};
use crate::{context::Context, request::{Request, PathParams}};

#[allow(non_snake_case)]
pub(crate) struct Handler<'router> {
    pub(crate) route:  &'static str,
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
