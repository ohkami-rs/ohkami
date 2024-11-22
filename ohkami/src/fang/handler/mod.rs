mod into_handler;
pub(crate) use into_handler::IntoHandler;

use super::{FangProcCaller, BoxedFPC};
use super::{SendOnNative, SendSyncOnNative, ResponseFuture};
use crate::{Request, Response};
use std::{pin::Pin, future::Future};


#[derive(Clone)]
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
    pub(crate) fn new(
        proc: impl Fn(&mut Request) -> Pin<Box<dyn ResponseFuture + '_>> + SendSyncOnNative + 'static
    ) -> Self {
        struct HandlerProc<F>(F);

        const _: () = {
            impl<F> FangProcCaller for HandlerProc<F>
            where
                F: Fn(&mut Request) -> Pin<Box<dyn ResponseFuture + '_>> + SendSyncOnNative + 'static
            {
                #[cfg(not(feature="rt_worker"))]
                fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
                    // SAFETY: trait upcasting
                    // trait upcasting coercion is experimental <https://github.com/rust-lang/rust/issues/65991>
                    unsafe {std::mem::transmute((self.0)(req))}
                }
                #[cfg(feature="rt_worker")]
                fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + 'b>> {
                    // SAFETY: trait upcasting
                    // trait upcasting coercion is experimental <https://github.com/rust-lang/rust/issues/65991>
                    unsafe {std::mem::transmute((self.0)(req))}
                }
            }
        };

        Self(BoxedFPC::from_proc(HandlerProc(proc)))
    }
}

#[cfg(feature="rt_worker")]
const _: () = {
    unsafe impl Send for Handler {}
    unsafe impl Sync for Handler {}
};

impl Handler {
    pub(crate) fn default_not_found() -> Self {
        use std::sync::LazyLock;

        static NOT_FOUND: LazyLock<Handler> = LazyLock::new(|| {
            async fn not_found() -> Response {
                Response::NotFound()
            }
            not_found.into_handler()
        });

        Handler((&*NOT_FOUND).0.clone())
    }

    pub(crate) fn default_options_with(mut available_methods: Vec<&'static str>) -> Self {
        let available_methods: &'static [&'static str] = {
            if available_methods.contains(&"GET") {
                available_methods.push("HEAD")
            }
            available_methods.push("OPTIONS");
            available_methods
        }.leak();

        let available_methods_str: &'static str =
            available_methods.join(", ").leak();

        Handler::new(move |req| {
            Box::pin(async move {
                #[cfg(debug_assertions)] {
                    assert_eq!(req.method, crate::Method::OPTIONS);
                }

                match req.headers.AccessControlRequestMethod() {
                    Some(method) => {
                        /*
                            Ohkami, by default, does nothing more than setting
                            `Access-Control-Allow-Methods` to preflight request.
                            CORS fang must override `Not Implemented` response,
                            whitch is the default for a valid preflight request,
                            by a successful one in its proc.
                        */
                        (if available_methods.contains(&method) {
                            crate::Response::NotImplemented()
                        } else {
                            crate::Response::BadRequest()
                        }).with_headers(|h| h
                            .AccessControlAllowMethods(available_methods_str)
                        )
                    }
                    None => {
                        /*
                            For security reasons, Ohkami doesn't support the
                            normal behavior to OPTIONS request like

                            ```
                            crate::Response::NoContent()
                                .with_headers(|h| h
                                    .Allow(available_methods_str)
                                )
                            ```
                        */
                        crate::Response::NotFound()
                    }
                }
            })
        })
    }
}
