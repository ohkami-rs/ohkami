mod into_handler;
pub(crate) use into_handler::IntoHandler;

use super::{FangProc, FangProcCaller, BoxedFPC};
use crate::{Request, Response};
use std::{pin::Pin, future::Future};


pub struct Handler(BoxedFPC);
const _: () = {
    impl FangProc for Handler {
        #[inline(always)]
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send + 'b {
            self.handle(req)
        }
        #[inline]
        fn bite_boxed<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
            self.handle(req)
        }
    }
};

impl Handler {
    pub(crate) fn new(proc: impl
        Fn(&mut Request) -> Pin<Box<dyn
            Future<Output = Response> + Send + '_
        >> + Send + Sync + 'static
    ) -> Self {
        struct HandlerProc<F: Fn(&mut Request) -> Pin<Box<dyn
            Future<Output = Response> + Send + '_
        >> + Send + Sync + 'static>(F);

        impl<F: Fn(&mut Request) -> Pin<Box<dyn
            Future<Output = Response> + Send + '_
        >> + Send + Sync + 'static>
        FangProcCaller for HandlerProc<F> {
            fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
                (self.0)(req)
            }
        }

        Self(BoxedFPC::from_proc(HandlerProc(proc)))
    }

    #[inline(always)]
    pub(crate) fn handle<'req>(
        &'req self,
        req: &'req mut Request,
    ) -> Pin<Box<dyn Future<Output = Response> + Send + 'req>> {
        self.0.call_bite(req)
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

    impl Into<BoxedFPC> for Handler {
        fn into(self) -> BoxedFPC {
            self.0
        }
    }
};
