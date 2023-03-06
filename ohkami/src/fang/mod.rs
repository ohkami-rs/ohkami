pub mod into_fang;
pub mod route;

use std::{pin::Pin, future::Future};
use crate::{context::Context, request::Request};
use self::into_fang::IntoFang;

pub struct Fangs<'req>(Vec<(
    &'static str/* route */,
    Fang<'req>,
)>);
pub type Fang<'req> =
    Box<dyn
        Fn(Context, Request<'req>) -> Pin<
            Box<dyn
                Future<Output = (Context, Request<'req>)>
                + Send
            >
        > + Send + Sync
    >
;

pub(crate) fn combine<'req>(this: &'req Fang<'req>, another: &'req Fang<'req>) -> Fang<'req> {
    Box::new(|c, request| Box::pin(async {
        (c, request) = this(c, request).await;
        (c, request) = another(c, request).await;
        (c, request)
    }))
}

impl<'req> Fangs<'req> {
    pub fn bite<const N: usize>(fangs: [dyn IntoFang<'req>; N]) -> Self {
        let mut fangs = vec![];

        Self(fangs)
    }
}
