#![cfg(any(feature="rt_tokio", feature="rt_async-std"))]

mod handlers;
pub use handlers::{Handlers, ByAnother, Dir, Route};

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

impl Handler {
    pub(crate) fn new(proc: impl
        Fn(&Request) -> Pin<Box<dyn
            Future<Output = Response>
            + Send + '_
        >>
        + Send + Sync
        + 'static
    ) -> Self {
        Self(Arc::new(proc))
    }

    #[inline(always)]
    pub(crate) fn handle<'req>(
        &'req self,
        req: &'req Request,
    ) -> Pin<Box<dyn Future<Output = Response> + Send + 'req>> {
        (self.0)(req)
    }
}

impl Handler {
    pub(crate) fn default_not_found() -> Self {
        async fn not_found() -> Response {
            Response::NotFound()
        }

        not_found.into_handler()
    }


    pub(crate) fn default_no_content() -> Self {
        async fn no_content() -> Response {
            Response::NoContent()
        }

        no_content.into_handler()
    }

    pub(crate) fn default_method_not_allowed() -> Self {
        async fn method_not_allowed() -> Response {
            Response::MethodNotAllowed()
        }

        method_not_allowed.into_handler()
    }
}

const _: () = {
    impl std::fmt::Debug for Handler {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("{handler}")
        }
    }
};
