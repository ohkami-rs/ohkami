pub mod into_handlefunc;

use std::{pin::Pin, future::Future};
use crate::{context::Context, request::Request};

#[allow(non_snake_case)]
pub(crate) struct Handler {
    route:  &'static str,
    GET:    Option<HandleFunc>,
    POST:   Option<HandleFunc>,
    PATCH:  Option<HandleFunc>,
    DELETE: Option<HandleFunc>,
}

pub(crate) type HandleFunc =
    Box<dyn
        Fn(Context, Request) -> Pin<
            Box<dyn
                Future<Output = crate::Result<()>>
                + Send
            >
        > + Send + Sync
    >
;
