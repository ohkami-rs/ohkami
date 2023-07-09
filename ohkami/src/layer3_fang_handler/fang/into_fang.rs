#![allow(non_snake_case)]

use std::{future::Future, sync::Arc, any::Any};
use super::{Fang};
use crate::{
    Context,
    Request, layer3_fang_handler::FrontFang,
};


pub trait IntoFang<Args, Output> {
    fn into_fang(self) -> Fang;
}

const _: (/* only Context */) = {
    impl<'req, F, Fut> IntoFang<(&Context,), Fut> for F
    where
        F:   Fn(&'req Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // SAFETY: `Fang::Front`s should be executed
        // **BEFORE** the handler by router
        fn into_fang(self) -> Fang {
            Fang::Front(FrontFang {
                id: self.type_id(),
                proc: Arc::new(move |c, req| Box::pin({
                    let out = unsafe {self(
                        std::mem::transmute::<_, &'req _>(&c)
                    )};
                    async {out.await; (c, req)}
                })),
            })
        }
    }
    impl<'req, F, Fut> IntoFang<(&mut Context,), Fut> for F
    where
        F:   Fn(&'req mut Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // SAFETY: `Fang::Front`s should be executed
        // **BEFORE** the handler by router
        fn into_fang(self) -> Fang {
            Fang::Front(FrontFang {
                id: self.type_id(),
                proc: Arc::new(move |mut c, req| Box::pin({
                    let out = unsafe {self(
                        std::mem::transmute::<_, &'req mut _>(&mut c)
                    )};
                    async {out.await; (c, req)}
                }))
            })
        }
    }
    impl<'req, F, Fut> IntoFang<((Context,), (Context,)), Fut> for F
    where
        F:   Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Context> + Send + 'static,
    {
        // SAFETY: `Fang::Front`s should be executed
        // **BEFORE** the handler by router
        fn into_fang(self) -> Fang {
            Fang::Front(FrontFang {
                id: self.type_id(),
                proc: Arc::new(move |c, req| Box::pin({
                    let new_c = self(c);
                    async {(new_c.await, req)}
                }))
            })
        }
    }
};

const _: (/* only Request */) = {
    impl<'req, F, Fut> IntoFang<(&Request,), Fut> for F
    where
        F:   Fn(&'req Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // SAFETY: `Fang::Front`s should be executed
        // **BEFORE** the handler by router
        fn into_fang(self) -> Fang {
            Fang::Front(FrontFang {
                id: self.type_id(),
                proc: Arc::new(move |c, req| Box::pin({
                    let out = unsafe {self(
                        std::mem::transmute::<_, &'req _>(&req)
                    )};
                    async {out.await; (c, req)}
                }))
            })
        }
    }
};

const _: (/* with Request */) = {
    impl<'req, F, Fut> IntoFang<(&Context, &Request), Fut> for F
    where
        F:   Fn(&'req Context, &'req Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // SAFETY: `Fang::Front`s should be executed
        // **BEFORE** the handler by router
        fn into_fang(self) -> Fang {
            Fang::Front(FrontFang {
                id: self.type_id(),
                proc: Arc::new(move |c, req| Box::pin({
                    let out = unsafe {self(
                        std::mem::transmute::<_, &'req _>(&c),
                        std::mem::transmute::<_, &'req _>(&req),
                    )};
                    async {out.await; (c, req)}
                }))
            })
        }
    }
    impl<'req, F, Fut> IntoFang<(&mut Context, &Request), Fut> for F
    where
        F:   Fn(&'req mut Context, &'req Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        // SAFETY: `Fang::Front`s should be executed
        // **BEFORE** the handler by router
        fn into_fang(self) -> Fang {
            Fang::Front(FrontFang {
                id: self.type_id(),
                proc: Arc::new(move |mut c, req| Box::pin({
                    let out = unsafe {self(
                        std::mem::transmute::<_, &'req mut _>(&mut c),
                        std::mem::transmute::<_, &'req _>(&req),
                    )};
                    async {out.await; (c, req)}
                }))
            })
        }
    }
};

#[cfg(test)] #[allow(unused)] const _: () = {
    async fn log(req: &Request) {
        println!("requested: {} {}", req.method(), req.path())
    }

    async fn add_server_header(c: &mut Context) {
        c.headers.Server("ohkami");
    }

    fn __() {
        let log_fang = log  .into_fang();
        let add_header_fang = add_server_header .into_fang();
    }
};
