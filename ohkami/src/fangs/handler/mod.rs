mod into_handler;
pub(crate) use into_handler::IntoHandler;

use super::{FangProcCaller, BoxedFPC};
use crate::{Request, Response};
use std::{pin::Pin, future::Future};


pub struct Handler(BoxedFPC);

const _: () = {
    impl Into<BoxedFPC> for Handler {
        fn into(self) -> BoxedFPC {
            self.0
        }
    }

    impl std::fmt::Debug for Handler {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("{handler}")
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

        const _: () = {
            impl<F: Fn(&mut Request) -> Pin<Box<dyn
                Future<Output = Response> + Send + '_
            >> + Send + Sync + 'static>
            FangProcCaller for HandlerProc<F> {
                fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
                    (self.0)(req)
                }
            }
        };

        Self(BoxedFPC::from_proc(HandlerProc(proc)))
    }

    pub(crate) fn __new__<
        Fut: Future<Output = Response> + Send
    >(
        proc: impl Fn(&mut Request) -> Fut + Send + Sync + 'static
    ) -> Self {
        

        todo!()
    }
}

impl Handler {
    pub(crate) fn default_not_found() -> Self {        
        Handler({
            static H: std::sync::OnceLock<Handler> = std::sync::OnceLock::new();
            H.get_or_init(|| {
                async fn not_found() -> Response {
                    Response::NotFound()
                }
                not_found.into_handler()
            }).0.clone()
        })
    }
    pub(crate) fn default_no_content() -> Self {
        Handler({
            static H: std::sync::OnceLock<Handler> = std::sync::OnceLock::new();
            H.get_or_init(|| {
                async fn not_found() -> Response {
                    Response::NoContent()
                }
                not_found.into_handler()
            }).0.clone()
        })
    }
    pub(crate) fn default_method_not_allowed() -> Self {
        Handler({
            static H: std::sync::OnceLock<Handler> = std::sync::OnceLock::new();
            H.get_or_init(|| {
                async fn not_found() -> Response {
                    Response::MethodNotAllowed()
                }
                not_found.into_handler()
            }).0.clone()
        })
    }
}
