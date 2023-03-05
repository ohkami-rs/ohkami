use std::{pin::Pin, future::Future};
use crate::{context::Context, request::Request};

pub struct Fangs<'router>(Vec<(
    &'static str/* route */,
    Fang<'router>,
)>);
pub(crate) type Fang<'router> =
    Box<dyn
        Fn(Context, Request<'router>) -> Pin<
            Box<dyn
                Future<Output = (Context, Request<'router>)>
                + Send
            >
        > + Send + Sync
    >
;

impl<'router> Fangs<'router> {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
}
