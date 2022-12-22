use std::{pin::Pin, future::Future};

use crate::context::Context;

type MiddlewareFunc = Box<dyn Fn(Context) -> Context + Send + Sync>;

pub struct Middleware(
    Vec<(/*route*/&'static str, MiddlewareFunc)>
); impl Middleware {
    pub fn init() -> Self {
        Self(Vec::new())
    }
}
