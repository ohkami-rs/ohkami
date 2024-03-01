#![cfg(any(feature="rt_tokio", feature="rt_async-std"))]

mod handlers;
pub use handlers::{Handlers, ByAnother, Route};

mod into_handler;
pub use into_handler::{IntoHandler};


use std::{
    pin::Pin,
    sync::Arc,
    future::Future,
};
use crate::{Request, Response};


#[derive(Clone)]
pub struct Handler(Arc<dyn
    Fn(&Request) -> Pin<Box<dyn
        Future<Output = Response>
        + Send + '_
    >>
    + Send + Sync
    + 'static
>);
const _: () = {
    impl Handler {
        fn new(proc: impl
            Fn(&Request) -> Pin<Box<dyn
                Future<Output = Response>
                + Send + '_
            >>
            + Send + Sync
            + 'static
        ) -> Self {
            Self(Arc::new(proc))
        }

        #[inline(always)] pub(crate) fn handle<'req>(
            &'req self,
            req: &'req Request,
        ) -> impl Future<Output = Response> + Send + 'req {
            (self.0)(req)
        }
    }
};
