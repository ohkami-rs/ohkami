use std::{pin::Pin, future::Future};
use crate::{handler::Handler, IntoResponse, Request, Response};


pub trait Fang<Inner: FangProc> {
    type Proc: FangProc;
    fn chain(&self, inner: Inner) -> Self::Proc;
}

pub trait FangProc: Sync {
    type Response: IntoResponse;
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Self::Response> + Send + 'b;
}


pub(crate) struct Inner(Box<dyn FangProcCaller + Send + Sync>);
const _: () = {
    impl Inner {
        pub(crate) fn from_proc(proc: impl FangProcCaller + Send + Sync + 'static) -> Self {
            Self(Box::new(proc))
        }
        pub(crate) fn from_proc_boxed(proc: Box<dyn FangProcCaller + Send + Sync>) -> Self {
            Self(proc)
        }
    }

    impl FangProc for Inner {
        type Response = Response;
        fn bite<'b>(&'b self, req: &'b mut Request) ->impl Future<Output = Response> + Send + 'b {
            self.0.bite_caller(req)
        }
    }
};


pub(crate) trait FangProcCaller {
    fn bite_caller<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>>;
}
impl<Proc: FangProc> FangProcCaller for Proc {
    fn bite_caller<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
        let res = self.bite(req);
        Box::pin(async move {res.await.into_response()})
    }
}
const _: () = {
    impl FangProcCaller for Handler {
        fn bite_caller<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
            self.handle(req)
        }
    }
};


#[allow(private_interfaces)]
pub trait Fangs {
    fn build(&self, inner: Inner) -> Box<dyn FangProcCaller + Send + Sync + 'static>;
    fn build_handler(&self, handler: Handler) -> Box<dyn FangProcCaller + Send + 'static>;
}
#[allow(private_interfaces)]
const _: () = {
    trait FangsCore {
        fn build_core(&self, inner: impl FangProcCaller + Send + Sync + 'static) -> impl FangProcCaller + Send + Sync + 'static;
    }
    impl<FC: FangsCore> Fangs for FC {
        fn build(&self, inner: Inner) -> Box<dyn FangProcCaller + Send + Sync + 'static> {
            Box::new(self.build_core(inner))
        }
        fn build_handler(&self, handler: Handler) -> Box<dyn FangProcCaller + Send + 'static> {
            Box::new(self.build_core(handler))
        }
    }


    impl FangsCore for () {
        fn build_core(&self, inner: impl FangProcCaller + Send + Sync + 'static) -> impl FangProcCaller + Send + Sync + 'static {
            inner
        }
    }

    impl<F> FangsCore for F
    where
        F: Fang<Inner>, F::Proc: Send + Sync + 'static,
    {
        fn build_core(&self, inner: impl FangProcCaller + Send + Sync + 'static) -> impl FangProcCaller + Send + Sync + 'static {
            self.chain(Inner::from_proc(inner))
        }
    }

    impl<F1> Fangs for (F1,)
    where
        F1: Fang<Inner>, F1::Proc: Send + Sync + 'static,
    {
        fn build(&self, inner: Inner) -> Box<dyn FangProcCaller + Send + Sync + 'static> {
            let (f1,) = self;
            Box::new(
                f1.chain(inner)
            )
        }
    }

    impl<F1, F2> Fangs for (F1, F2)
    where
        F1: Fang<F2::Proc>, F1::Proc: Send + Sync + 'static,
        F2: Fang<Inner>,    F2::Proc: Send + Sync + 'static,
    {
        fn build(&self, inner: Inner) -> Box<dyn FangProcCaller + Send + Sync + 'static> {
            let (f1, f2) = self;
            Box::new(
                f1.chain(
                    f2.chain(inner)
                )
            )
        }
    }

    impl<F1, F2, F3> Fangs for (F1, F2, F3)
    where
        F1: Fang<F2::Proc>, F1::Proc: Send + Sync + 'static,
        F2: Fang<F3::Proc>, F2::Proc: Send + Sync + 'static,
        F3: Fang<Inner>,    F3::Proc: Send + Sync + 'static,
    {
        fn build(&self, inner: Inner) -> Box<dyn FangProcCaller + Send + Sync + 'static> {
            let (f1, f2, f3) = self;
            Box::new(
                f1.chain(
                    f2.chain(
                        f3.chain(inner)
                    )
                )
            )
        }
    }

    impl<F1, F2, F3, F4> Fangs for (F1, F2, F3, F4)
    where
        F1: Fang<F2::Proc>, F1::Proc: Send + Sync + 'static,
        F2: Fang<F3::Proc>, F2::Proc: Send + Sync + 'static,
        F3: Fang<F4::Proc>, F3::Proc: Send + Sync + 'static,
        F4: Fang<Inner>,    F4::Proc: Send + Sync + 'static,
    {
        fn build(&self, inner: Inner) -> Box<dyn FangProcCaller + Send + Sync + 'static> {
            let (f1, f2, f3, f4) = self;
            Box::new(
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

    impl<F1, F2, F3, F4, F5> Fangs for (F1, F2, F3, F4, F5)
    where
        F1: Fang<F2::Proc>, F1::Proc: Send + Sync + 'static,
        F2: Fang<F3::Proc>, F2::Proc: Send + Sync + 'static,
        F3: Fang<F4::Proc>, F3::Proc: Send + Sync + 'static,
        F4: Fang<F5::Proc>, F4::Proc: Send + Sync + 'static,
        F5: Fang<Inner>,    F5::Proc: Send + Sync + 'static,
    {
        fn build(&self, inner: Inner) -> Box<dyn FangProcCaller + Send + Sync + 'static> {
            let (f1, f2, f3, f4, f5) = self;
            Box::new(
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

    impl<F1, F2, F3, F4, F5, F6> Fangs for (F1, F2, F3, F4, F5, F6)
    where
        F1: Fang<F2::Proc>, F1::Proc: Send + Sync + 'static,
        F2: Fang<F3::Proc>, F2::Proc: Send + Sync + 'static,
        F3: Fang<F4::Proc>, F3::Proc: Send + Sync + 'static,
        F4: Fang<F5::Proc>, F4::Proc: Send + Sync + 'static,
        F5: Fang<F6::Proc>, F5::Proc: Send + Sync + 'static,
        F6: Fang<Inner>,    F6::Proc: Send + Sync + 'static,
    {
        fn build(&self, inner: Inner) -> Box<dyn FangProcCaller + Send + Sync + 'static> {
            let (f1, f2, f3, f4, f5, f6) = self;
            Box::new(
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

    impl<F1, F2, F3, F4, F5, F6, F7> Fangs for (F1, F2, F3, F4, F5, F6, F7)
    where
        F1: Fang<F2::Proc>, F1::Proc: Send + Sync + 'static,
        F2: Fang<F3::Proc>, F2::Proc: Send + Sync + 'static,
        F3: Fang<F4::Proc>, F3::Proc: Send + Sync + 'static,
        F4: Fang<F5::Proc>, F4::Proc: Send + Sync + 'static,
        F5: Fang<F6::Proc>, F5::Proc: Send + Sync + 'static,
        F6: Fang<F7::Proc>, F6::Proc: Send + Sync + 'static,
        F7: Fang<Inner>,    F7::Proc: Send + Sync + 'static,
    {
        fn build(&self, inner: Inner) -> Box<dyn FangProcCaller + Send + Sync + 'static> {
            let (f1, f2, f3, f4, f5, f6, f7) = self;
            Box::new(
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

    impl<F1, F2, F3, F4, F5, F6, F7, F8> Fangs for (F1, F2, F3, F4, F5, F6, F7, F8)
    where
        F1: Fang<F2::Proc>, F1::Proc: Send + Sync + 'static,
        F2: Fang<F3::Proc>, F2::Proc: Send + Sync + 'static,
        F3: Fang<F4::Proc>, F3::Proc: Send + Sync + 'static,
        F4: Fang<F5::Proc>, F4::Proc: Send + Sync + 'static,
        F5: Fang<F6::Proc>, F5::Proc: Send + Sync + 'static,
        F6: Fang<F7::Proc>, F6::Proc: Send + Sync + 'static,
        F7: Fang<F8::Proc>, F7::Proc: Send + Sync + 'static,
        F8: Fang<Inner>,    F8::Proc: Send + Sync + 'static,
    {
        fn build(&self, inner: Inner) -> Box<dyn FangProcCaller + Send + Sync + 'static> {
            let (f1, f2, f3, f4, f5, f6, f7, f8) = self;
            Box::new(
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
