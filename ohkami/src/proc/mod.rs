pub(crate) mod fang;
pub(crate) mod handler;

use std::{future::Future, pin::Pin};
use crate::{Request, Response};


pub(crate) trait Proc {
    fn call<'p>(&'p self, req: &'p mut Request) -> impl Future<Output = Response> + Send + 'p;
}

/// object-safe `Proc` returning `Pin<Box<dyn Future>>`
pub(crate) trait BoxedProc {
    fn call_boxed<'p>(&'p self, req: &'p mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'p>>;
}
impl<P: Proc> BoxedProc for P {
    fn call_boxed<'p>(&'p self, req: &'p mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'p>> {
        Box::pin(self.call(req))
    }
}
