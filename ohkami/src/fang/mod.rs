use std::{future::Future, ops::Deref, pin::Pin};
use crate::{handler::Handler, IntoResponse, Request, Response};


pub trait Fang<Inner: FangProc> {
    type Proc: FangProc;
    fn chain(&self, inner: Inner) -> Self::Proc;
}
pub trait FangProc: Send + Sync + 'static {
    type Response: IntoResponse;
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Self::Response> + Send + 'b;
}


pub(crate) trait FangProcCaller {
    fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>>;
}

pub(crate) struct BoxedFPC(Box<dyn
    FangProcCaller + Send + Sync + 'static
>);
const _: () = {
    impl Deref for BoxedFPC {
        type Target = dyn FangProcCaller + Send + Sync + 'static;
        
        #[inline(always)]
        fn deref(&self) -> &Self::Target {
            &*self.0
        }
    }

    impl std::fmt::Debug for BoxedFPC {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("{Fang proc}")
        }
    }
};
impl BoxedFPC {
    pub(crate) fn from_proc(proc: impl FangProcCaller + Send + Sync + 'static) -> Self {
        Self(Box::new(proc))
    }
}

const _: () = {
    impl FangProc for Handler {
        type Response = Response;

        #[inline(always)]
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Self::Response> + Send + 'b {
            self.handle(req)  // Pin<Box<dyn Future>>
        }
    }

    impl FangProc for BoxedFPC {
        type Response = Response;

        #[inline(always)]
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Self::Response> + Send + 'b {
            (&*self.0).call_bite(req)  // Pin<Box<dyn Future>>
        }
    }
};

#[cfg(not(feature="nightly"))]
const _: () = {
    impl<Proc: FangProc> FangProcCaller for Proc {
        #[inline(always)]
        fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
            let res = self.bite(req);
            Box::pin(async move {res.await.into_response()})
        }
    }
};
#[cfg(feature="nightly")]
const _: () = {
    impl<Proc: FangProc> FangProcCaller for Proc {
        #[inline(always)]
        default fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
            let res = self.bite(req);
            Box::pin(async move {res.await.into_response()})
        }
    }

    impl FangProcCaller for Handler {
        #[inline(always)]
        fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
            // omit doubly-boxed future
            self.handle(req)
        }
    }

    impl FangProcCaller for BoxedFPC {
        #[inline(always)]
        fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
            // omit doubly-boxed future
            (&*self.0).call_bite(req)
        }
    }
};


#[allow(private_interfaces)]
pub trait Fangs {
    // returning box for object-safety
    fn build(&self, inner: BoxedFPC) -> BoxedFPC;
    // specialize to omit doubly dynamic dispatching by `Inner`
    fn build_handler(&self, handler: Handler) -> BoxedFPC;
}
#[allow(private_interfaces)]
const _: () = {
    /*===== tuple impl helper =====*/

    trait FangsHelper<Inner: FangProc> {
        fn build_helper(&self, inner: Inner) -> BoxedFPC;
    }
    impl<FH: FangsHelper<BoxedFPC> + FangsHelper<Handler>> Fangs for FH {
        fn build(&self, inner: BoxedFPC) -> BoxedFPC {
            <Self as FangsHelper<BoxedFPC>>::build_helper(&self, inner)
        }
        fn build_handler(&self, handler: Handler) -> BoxedFPC {
            <Self as FangsHelper<Handler>>::build_helper(&self, handler)
        }
    }


    /*===== tuple impls =====*/

    impl<Inner: FangProc> FangsHelper<Inner> for () {
        fn build_helper(&self, inner: Inner) -> BoxedFPC {
            BoxedFPC::from_proc(inner)
        }
    }

    impl<Inner: FangProc,
        F1: Fang<Inner>,
    > FangsHelper<Inner> for (F1,)
    where
    {
        fn build_helper(&self, inner: Inner) -> BoxedFPC {
            let (f1,) = self;
            BoxedFPC::from_proc(
                f1.chain(inner)
            )
        }
    }

    impl<Inner: FangProc,
        F1: Fang<F2::Proc>,
        F2: Fang<Inner>,
    > FangsHelper<Inner> for (F1, F2) {
        fn build_helper(&self, inner: Inner) -> BoxedFPC {
            let (f1, f2) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(inner)
                )
            )
        }
    }

    impl<Inner: FangProc,
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<Inner>,
    > FangsHelper<Inner> for (F1, F2, F3) {
        fn build_helper(&self, inner: Inner) -> BoxedFPC {
            let (f1, f2, f3) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(inner)
                    )
                )
            )
        }
    }

    impl<Inner: FangProc,
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<F4::Proc>,
        F4: Fang<Inner>,
    > FangsHelper<Inner> for (F1, F2, F3, F4) {
        fn build_helper(&self, inner: Inner) -> BoxedFPC {
            let (f1, f2, f3, f4) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(
                            f4.chain(inner)
                        )
                    )
                )
            )
        }
    }

    impl<Inner: FangProc,
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<F4::Proc>,
        F4: Fang<F5::Proc>,
        F5: Fang<Inner>,
    > FangsHelper<Inner> for (F1, F2, F3, F4, F5) {
        fn build_helper(&self, inner: Inner) -> BoxedFPC {
            let (f1, f2, f3, f4, f5) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(
                            f4.chain(
                                f5.chain(inner)
                            )
                        )
                    )
                )
            )
        }
    }

    impl<Inner: FangProc,
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<F4::Proc>,
        F4: Fang<F5::Proc>,
        F5: Fang<F6::Proc>,
        F6: Fang<Inner>,
    > FangsHelper<Inner> for (F1, F2, F3, F4, F5, F6) {
        fn build_helper(&self, inner: Inner) -> BoxedFPC {
            let (f1, f2, f3, f4, f5, f6) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(
                            f4.chain(
                                f5.chain(
                                    f6.chain(inner)
                                )
                            )
                        )
                    )
                )
            )
        }
    }

    impl<Inner: FangProc,
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<F4::Proc>,
        F4: Fang<F5::Proc>,
        F5: Fang<F6::Proc>,
        F6: Fang<F7::Proc>,
        F7: Fang<Inner>,
    > FangsHelper<Inner> for (F1, F2, F3, F4, F5, F6, F7) {
        fn build_helper(&self, inner: Inner) -> BoxedFPC {
            let (f1, f2, f3, f4, f5, f6, f7) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(
                            f4.chain(
                                f5.chain(
                                    f6.chain(
                                        f7.chain(inner)
                                    )
                                )
                            )
                        )
                    )
                )
            )
        }
    }

    impl<Inner: FangProc,
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<F4::Proc>,
        F4: Fang<F5::Proc>,
        F5: Fang<F6::Proc>,
        F6: Fang<F7::Proc>,
        F7: Fang<F8::Proc>,
        F8: Fang<Inner>,
    > FangsHelper<Inner> for (F1, F2, F3, F4, F5, F6, F7, F8) {
        fn build_helper(&self, inner: Inner) -> BoxedFPC {
            let (f1, f2, f3, f4, f5, f6, f7, f8) = self;
            BoxedFPC::from_proc(
                f1.chain(
                    f2.chain(
                        f3.chain(
                            f4.chain(
                                f5.chain(
                                    f6.chain(
                                        f7.chain(
                                            f8.chain(inner)
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        }
    }
};
