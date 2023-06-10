#![allow(non_snake_case)]

use std::{future::Future};
use super::Fang;
use crate::{
    Context,
    Request,
};


pub trait IntoFang {
    fn into_fang(self) -> Fang;
}
impl<F, Fut> IntoFang for F
where
    F:   Fn(Context, Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = (Context, Request)> + Send + 'static,
{
    fn into_fang(self) -> Fang {
        Fang::Before(Box::new(move |c, req|
            Box::pin(self(c, req))
        ))
    }
}
/*

<
    &for<'a> fn(Context, &'a Request) -> impl Future<Output = Context> {fang_1}
    as
    FnOnce<(Context, &'req Request)>
>::Output = _

*/
#[cfg(test)] const _: () = {
    async fn fang_1(c: Context, req: Request) -> (Context, Request) {
        (c, req)
    }

    fn __() {
        let _ = fang_1.into_fang();
    }
};

const _: (/* Before */) = {
    // const _: (/* only Context */) = {
    //     impl<F, Fut> IntoFang<(&Context,)> for F
    //     where
    //         F:   Fn(&Context) -> Fut + Send + Sync + 'static,
    //         Fut: Future<Output = ()> + Send + 'static,
    //     {
    //         fn into_fang(self) -> Fang {
    //             Fang::Before(Box::new(move |c, req|
    //                 Box::pin({
    //                     let out = self(&c);
    //                     async {out.await; (c, req)}
    //                 })
    //             ))
    //         }
    //     }
// 
    //     impl<F, Fut> IntoFang<(&mut Context,)> for F
    //     where
    //         F:   Fn(&mut Context) -> Fut + Send + Sync + 'static,
    //         Fut: Future<Output = ()> + Send + 'static,
    //     {
    //         fn into_fang(self) -> Fang {
    //             Fang::Before(Box::new(move |mut c, req|
    //                 Box::pin({
    //                     let out = self(&mut c);
    //                     async {out.await; (c, req)}
    //                 })
    //             ))
    //         }
    //     }
    // };

    const _: (/* with Request */) = {
        // impl<'c, 'req, F, Fut> IntoFang<(&'c Context, &'req Request)> for F
        // where
        //     F:   Fn(&'c Context, &'req Request) -> Fut + Send + Sync + 'static,
        //     Fut: Future<Output = ()> + Send + 'static,
        // {
        //     fn into_fang(self) -> Fang {
        //         Fang::Before(Box::new(|c, req|
        //             Box::pin({
        //                 let out = self(&c, &req);
        //                 async move {out.await; (c, req)}
        //             })
        //         ))
        //     }
        // }

        // impl<F, Fut> IntoFang<(&mut Context, &Request)> for F
        // where
        //     F:   Fn(&mut Context, &Request) -> Fut + Send + Sync + 'static,
        //     Fut: Future<Output = ()> + Send + 'static,
        // {
        //     fn into_fang(self) -> Fang {
        //         Fang::Before(Box::new(move |mut c, req|
        //             Box::pin({
        //                 let out = self(&mut c, &req);
        //                 async {out.await; (c, req)}
        //             })
        //         ))
        //     }
        // }
    };
};


#[cfg(test)] #[allow(unused)] const _: () = {
    // async fn fang_1(c: &Context, _: &Request) {
    //     todo!()
    // }
// 
    // async fn fang_2(c: &mut Context, _: &Request) {
    //     todo!()
    // }
// 
    // fn __() {
    //     let _ = fang_1.into_fang();
    // }
};
