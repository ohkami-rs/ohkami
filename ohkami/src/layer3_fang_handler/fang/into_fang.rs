#![allow(non_snake_case)]

use std::{future::Future};
use super::{Fang};
use crate::{
    Context,
    Request,
};


pub trait FrontFang<Output: Future<Output = ()>> {
    fn into_fang(self) -> Fang;
}

impl<'req, F, Fut> FrontFang<Fut> for F
where
    F:   Fn(&'req Context, &'req Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    // SAFETY: `Fang::Front`s should be executed
    // **BEFORE** the handler by router
    fn into_fang(self) -> Fang {
        Fang::Before(Box::new(move |c, req| Box::pin({
            let out = unsafe {self(
                std::mem::transmute::<_, &'req _>(&c),
                std::mem::transmute::<_, &'req _>(&req),
            )};
            async {out.await; (c, req)}
        })))
    }
}

#[cfg(test)] #[allow(unused)] const _: () = {
    async fn log(_: &Context, req: &Request) {
        println!("requested: {} {}", req.method(), req.path())
    }

    fn __() {
        let log_fang = log.into_fang();
    }
};
