pub mod into_handlefunc;

use std::{pin::Pin, future::Future};
use crate::{router::route::HandlerRoute, context::Context, request::Request};

#[allow(non_snake_case)]
pub(crate) struct Handler<'buf> {
    route:  HandlerRoute,
    GET:    Option<HandleFunc<'buf>>,
    POST:   Option<HandleFunc<'buf>>,
    PATCH:  Option<HandleFunc<'buf>>,
    DELETE: Option<HandleFunc<'buf>>,
}

pub(crate) type HandleFunc<'buf> =
    Box<dyn
        Fn(Context, Request<'buf>) -> Pin<
            Box<dyn
                Future<Output = crate::Result<()>>
                + Send
            >
        > + Send + Sync
    >
;
