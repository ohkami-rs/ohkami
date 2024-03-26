use std::{pin::Pin, sync::Arc, future::Future};
use crate::{handler::Handler, IntoResponse, Request, Response};


pub trait Fang<Inner: FangProc> {
    type Proc: FangProc;
    fn chain(&self, inner: Inner) -> Self::Proc;
}

pub trait FangProc: Sync {
    type Response: IntoResponse;
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Self::Response> + Send + 'b;
}


struct Inner(Box<dyn FangProcCaller + Send + Sync>);
const _: () = {
    impl Inner {
        pub(crate) fn new() -> Self {
            struct NoneFangCaller;
            impl FangProcCaller for NoneFangCaller {
                fn bite_caller<'b>(&'b self, _: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
                    unreachable!("NoneFangCaller::bite_caller")
                }
            }
        
            Self(Box::new(NoneFangCaller))
        }

        pub(crate) fn from_proc(proc: impl FangProcCaller + Send + Sync + 'static) -> Self {
            Self(Box::new(proc))
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


pub trait Fangs {
    fn build(&self, inner: Inner) -> Arc<dyn FangProcCaller>;
} const _: () = {
    impl Fangs for () {
        fn build(&self, inner: Inner) -> Arc<dyn FangProcCaller> {
            Arc::new(inner)
        }
    }

    impl<F1> Fangs for (F1,)
    where
        F1: Fang<Inner>, F1::Proc: 'static,
    {
        fn build(&self, inner: Inner) -> Arc<dyn FangProcCaller> {
            let (f1,) = self;
            Arc::new(
                f1.chain(inner)
            )
        }
    }

    impl<F1, F2> Fangs for (F1, F2)
    where
        F1: Fang<F2::Proc>, F1::Proc: 'static,
        F2: Fang<Inner>,    F2::Proc: 'static,
    {
        fn build(&self, inner: Inner) -> Arc<dyn FangProcCaller> {
            let (f1, f2) = self;
            Arc::new(
                f1.chain(
                    f2.chain(inner)
                )
            )
        }
    }

    impl<F1, F2, F3> Fangs for (F1, F2, F3)
    where
        F1: Fang<F2::Proc>, F1::Proc: 'static,
        F2: Fang<F3::Proc>, F2::Proc: 'static,
        F3: Fang<Inner>,    F3::Proc: 'static,
    {
        fn build(&self, inner: Inner) -> Arc<dyn FangProcCaller> {
            let (f1, f2, f3) = self;
            Arc::new(
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
        F1: Fang<F2::Proc>, F1::Proc: 'static,
        F2: Fang<F3::Proc>, F2::Proc: 'static,
        F3: Fang<F4::Proc>, F3::Proc: 'static,
        F4: Fang<Inner>,    F4::Proc: 'static,
    {
        fn build(&self, inner: Inner) -> Arc<dyn FangProcCaller> {
            let (f1, f2, f3, f4) = self;
            Arc::new(
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
        F1: Fang<F2::Proc>, F1::Proc: 'static,
        F2: Fang<F3::Proc>, F2::Proc: 'static,
        F3: Fang<F4::Proc>, F3::Proc: 'static,
        F4: Fang<F5::Proc>, F4::Proc: 'static,
        F5: Fang<Inner>,    F5::Proc: 'static,
    {
        fn build(&self, inner: Inner) -> Arc<dyn FangProcCaller> {
            let (f1, f2, f3, f4, f5) = self;
            Arc::new(
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
        F1: Fang<F2::Proc>, F1::Proc: 'static,
        F2: Fang<F3::Proc>, F2::Proc: 'static,
        F3: Fang<F4::Proc>, F3::Proc: 'static,
        F4: Fang<F5::Proc>, F4::Proc: 'static,
        F5: Fang<F6::Proc>, F5::Proc: 'static,
        F6: Fang<Inner>,    F6::Proc: 'static,
    {
        fn build(&self, inner: Inner) -> Arc<dyn FangProcCaller> {
            let (f1, f2, f3, f4, f5, f6) = self;
            Arc::new(
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
        F1: Fang<F2::Proc>, F1::Proc: 'static,
        F2: Fang<F3::Proc>, F2::Proc: 'static,
        F3: Fang<F4::Proc>, F3::Proc: 'static,
        F4: Fang<F5::Proc>, F4::Proc: 'static,
        F5: Fang<F6::Proc>, F5::Proc: 'static,
        F6: Fang<F7::Proc>, F6::Proc: 'static,
        F7: Fang<Inner>,    F7::Proc: 'static,
    {
        fn build(&self, inner: Inner) -> Arc<dyn FangProcCaller> {
            let (f1, f2, f3, f4, f5, f6, f7) = self;
            Arc::new(
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
        F1: Fang<F2::Proc>, F1::Proc: 'static,
        F2: Fang<F3::Proc>, F2::Proc: 'static,
        F3: Fang<F4::Proc>, F3::Proc: 'static,
        F4: Fang<F5::Proc>, F4::Proc: 'static,
        F5: Fang<F6::Proc>, F5::Proc: 'static,
        F6: Fang<F7::Proc>, F6::Proc: 'static,
        F7: Fang<F8::Proc>, F7::Proc: 'static,
        F8: Fang<Inner>,    F8::Proc: 'static,
    {
        fn build(&self, inner: Inner) -> Arc<dyn FangProcCaller> {
            let (f1, f2, f3, f4, f5, f6, f7, f8) = self;
            Arc::new(
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
