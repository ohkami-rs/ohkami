pub mod into_handlefunc;

use std::{pin::Pin, future::Future};
use async_std::net::TcpStream;

use crate::{context::Context, request::Request};

#[allow(non_snake_case)]
pub(crate) struct Handler<'router> {
    route:  &'static str,
    GET:    Option<HandleFunc<'router>>,
    POST:   Option<HandleFunc<'router>>,
    PATCH:  Option<HandleFunc<'router>>,
    DELETE: Option<HandleFunc<'router>>,
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
