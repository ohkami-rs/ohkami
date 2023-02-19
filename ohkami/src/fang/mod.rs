use std::{pin::Pin, future::Future};
use crate::{router::route::FangRoute, response::err::ErrResponse, context::Context, request::{Request, parse::method::Method}};

pub struct Fangs<'buf>(Vec<(
    Method,
    FangRoute,
    Fang<'buf>,
)>);
pub(crate) type Fang<'buf> =
    Box<dyn
        Fn(Context, Request<'buf>) -> Pin<
            Box<dyn
                Future<Output = Result<(Context, Request), ErrResponse>>
                + Send
            >
        > + Send + Sync
    >
;

impl<'buf> Fangs<'buf> {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
}
