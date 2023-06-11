#![allow(non_snake_case)]

use std::{future::Future, pin::Pin};
use super::Fang;
use crate::{
    Context,
    Request,
};


// pub trait BeforeFang<Output> {
//     fn bite(self, c: Context, req: Request) -> Pin<Box<dyn Future<Output = (Context, Request)>>>;
// }
// impl<F, Fut> BeforeFang for F
// where
//     F:   Fn(Context, Request) -> Fut + Send + Sync + 'static,
//     Fut: Future<Output = (Context, Request)> + Sync + 'static,
// {
//     type Output = Fut;
//     fn bite(self, c: Context, req: Request) -> Self::Output {
//         self(c, req)
//     }
// }
// impl<F, Fut> BeforeFang<Fut> for F
// where
//     F:   for<'c, 'req> Fn(&'c Context, &'req Request) -> Fut + Send + Sync + 'static,
//     Fut: Future<Output = ()> + Sync + 'static,
// {
//     // fn bite(self, c: Context, req: Request) -> Fut {
//     //     Box::pin(async move {self(&c, &req).await})
//     // }
//     fn bite(self, c: Context, req: Request) -> Pin<Box<dyn Future<Output = (Context, Request)>>> {
//         let out = self(&c, &req);
//         Box::pin(async {out.await; (c, req)})
//     }
// }

#[cfg(test)] const _: () = {
    // async fn fang_1(c: Context, req: Request) -> (Context, Request) {
    //     (c, req)
    // }
    async fn fang_2(_: &Context, _: &Request) {
    }
// 
    //fn __(c: Context, req: Request) -> (Context, Request) {
    //   fang_2.bite(c, req)
    //}
};

// trait FrontFang<Output: Future<Output = ()>> {
//     fn bite(self, c: &Context, req: &Request) -> Pin<Box<dyn Future<Output = ()>>>;
// }
// impl<'c, 'req, F, Fut> FrontFang<Fut> for F
// where
//     F:   Fn(&'c Context, &'req Request) -> Fut + Send + Sync + 'static,
//     Fut: Future<Output = ()> + Sync + 'static,
// {
//     // fn bite(self, c: Context, req: Request) -> Pin<Box<dyn Future<Output = (Context, Request)>>> {
//     //     // let output = self(&c, &req);
//     //     // Box::pin(async {output.await; (c, req)})
//     //     todo!()
//     // }
//     // fn bite(self, c: &Context, req: &Request) -> Pin<Box<dyn Future<Output = ()>>> {
//     //     Box::pin(self(c, req))
//     // }
//     fn bite(self, c: &Context, req: &Request) -> Pin<Box<dyn Future<Output = ()>>> {
//         // Box::pin(self(c, req))
//         todo!()
//     }
// }
// 
// #[cfg(test)] const _: () = {
//     async fn f(_: &Context, _: &Request) {}
// 
//     fn __(_: impl
//         for<'c, 'req> FnOnce<(&'c Context, &'req Request)>
//     ) {}
// 
//     fn ___(c: Context, req: Request) {
//         __(f);
//         let _ = f.bite(&c, &req);
//     }
// 
//     // fn __(c: &Context, req: &Request) {
//     //     let _ = f.bite(c, req);
//     // }
// };

pub trait FrontFang<Output: Future<Output = ()>> {
    fn into_fang(self) -> Fang;
}
impl<'c, 'req, F, Fut> FrontFang<Fut> for F
where
    F:   Fn(&'c Context, &'req Request) -> Fut,
    Fut: Future<Output = ()>
{
    fn into_fang(self) -> Fang {
        Fang::Before(Box::new(move |c, req| Box::pin({
            async {todo!(compile_error!("\
                Maybe, MAYBE, it should be safe to transmute `c`, `req` \
                into the desired lifetime here. \
                This is beased on my understanding: \n\n\
                    \"All `Fang::Front`s should be executed \
                    BEFORE the handler (this consumes `c` and `req`) does.\" \n\n\
                (If CPU executes the handler before a `Fang::Front`, \
                `&Context` and `&Request` just get invalid and then \
                executing the `Fang::Front` is UNDEFINED BEHAVIOR.) \n\
                But, I don't have full confidence that my understanding is correct...
            "))}
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

// 
// impl<F, Fut> IntoFang for F
// where
//     F:   Fn(Context, Request) -> Fut + Send + Sync + 'static,
//     Fut: Future<Output = (Context, Request)> + Send + 'static,
// {
//     fn into_fang(self) -> Fang {
//         Fang::Before(Box::new(move |c, req|
//             Box::pin(self(c, req))
//         ))
//     }
// }
// impl<'c, F, Fut> IntoFang<Fut> for F
// where
//     F:   Fn<(&'c Context, Request), Output = Fut> + Sync + Send + 'static,
//     Fut: Future<Output = Request> + Send + 'static
// {
//     fn into_fang(self) -> Fang {
//         Fang::Before(Box::new(move |c, req| Box::pin({
//             let req = self(&c, req);
//             async move {(c, req.await)}
//             // async {todo!()}
//         })))
//     }
// }
// 
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
