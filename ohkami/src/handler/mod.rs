pub mod into_handlefunc;

use async_std::net::TcpStream;
use std::{pin::Pin, future::Future};
use crate::{context::Context, request::Request};

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
        Fn(TcpStream, Context, Request<'router>) -> Pin<
            Box<dyn
                Future<Output = crate::Result<()>>
                + Send
            >
        > + Send + Sync
    >
;
