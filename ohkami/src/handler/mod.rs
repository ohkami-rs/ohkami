use std::{pin::Pin, future::Future};

use serde::Serialize;

use crate::{router::route::HandlerRoute, context::Context, request::Request, response::Response};

#[allow(non_snake_case)]
pub(crate) struct Handler<'buf> {
    route:  HandlerRoute,
    GET:    Option<HandleFunc<'buf>>,
    POST:   Option<HandleFunc<'buf>>,
    PATCH:  Option<HandleFunc<'buf>>,
    DELETE: Option<HandleFunc<'buf>>,
}

pub(crate) struct HandleFunc<'buf>(
    Box<dyn
        Fn(Context, Request<'buf>) -> Pin<
            Box<dyn
                Future<Output = crate::Result<()>>
                + Send
            >
        > + Send + Sync
    >
);


pub trait IntoHandler<'buf> {
    fn into_handler(self) -> Handler<'buf>;
}

impl<'buf, F, Fut, T> IntoHandler<'buf> for F
where
    F:   Fn(Context) -> Fut,
    Fut: Future<Output = Response<T>>,
    T:   Serialize,
{
    fn into_handler(self) -> Handler<'buf> {
        
    }
}

