use std::{pin::Pin, future::Future};
use crate::{response::ErrResponse, context::Context, request::{Request, parse::method::Method}};

pub struct Fangs(Vec<(
    Method,
    &'static str,
    Fang,
)>);
pub(crate) type Fang =
    Box<dyn
        Fn(Context, Request) -> Pin<
            Box<dyn
                Future<Output = Result<(Context, Request), ErrResponse>>
                + Send
            >
        > + Send + Sync
    >
;

impl Fangs {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
}
