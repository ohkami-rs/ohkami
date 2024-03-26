use std::{pin::Pin, sync::Arc, future::Future};
use crate::{handler::Handler, IntoResponse, Request, Response};


pub trait Fang<Inner: FangProc> {
    type Proc: FangProc;
    fn chain(self, inner: Inner) -> Self::Proc;
}

pub trait FangProc {
    type Response: IntoResponse;
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Self::Response> + Send + 'b;
}


struct Inner(Box<dyn FangProcCaller>);
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

        pub(crate) fn from_proc(inner: Inner) -> Self {
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
    fn build(self, inner: Inner) -> Arc<dyn FangProcCaller>;
} const _: () = {
    impl Fangs for () {
        fn build(self, inner: Inner) -> Arc<dyn FangProcCaller> {
            Arc::new(proc)
        }
    }

    impl<F1> Fangs for (F1,)
    where
        F1: Fang<Inner>,
        F1::Proc: 'static,
    {
        fn build(self, inner: Inner) -> Arc<dyn FangProcCaller> {
            let (f1,) = self;
            Arc::new(f1.chain(Inner::from_proc(proc)))
        }
    }

    impl<F1, F2> Fangs for (F1, F2)
    where
        F1: Fang<Inner>,
        F2: Fang<Inner>,
        F1::Proc: 'static,
        F2::Proc: 'static,
    {
        fn build(self, inner: Inner) -> Arc<dyn FangProcCaller> {
            let (f1, f2) = self;
            Arc::new(
                f1.chain(Inner::from_proc(
                    f2.chain(Inner::from_proc(proc))
                ))
            )
        }
    }
};


// macro_rules! tuple_fangs {
//     ($( $fang:ident ),*) => {
//         #[allow(non_snake_case)]
//         impl<$( $fang: Fang ),*> Fangs for ( $( $fang, )* ) {
//             fn collect(self) -> Vec<Arc<dyn FangProcCaller>> {
//                 let ( $( $fang, )* ) = self;
//                 vec![ $( Arc::new($fang) ),* ]
//             }
//         }
//     };
// } const _: () = {
//     tuple_fangs!();
//     tuple_fangs!(F1);
//     tuple_fangs!(F1, F2);
//     tuple_fangs!(F1, F2, F3);
//     tuple_fangs!(F1, F2, F3, F4);
//     tuple_fangs!(F1, F2, F3, F4, F5);
//     tuple_fangs!(F1, F2, F3, F4, F5, F6);
//     tuple_fangs!(F1, F2, F3, F4, F5, F6, F7);
// };


/* 
pub trait Fang<Inner: FangProc> {
    type Proc: FangProc;
    fn chain(self, inner: Inner) -> Self::Proc;
}

pub trait FangProc {
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send + 'b;
}
const _: () = {
    impl FangProc for Handler {
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send + 'b {
            self.handle(req)
        }
    }
};
trait FangProcCaller {
    fn call<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn std::future::Future<Output = Response> + Send + 'b>>;
}
impl<FP: FangProc> FangProcCaller for FP {
    fn call<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn std::future::Future<Output = Response> + Send + 'b>>{
        Box::pin(self.bite(req))
    }
}

#[derive(Clone)]
pub struct Inner(Arc<dyn FangProcCaller>);
impl FangProcCaller for Inner {
    fn call<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn std::future::Future<Output = Response> + Send + 'b>> {
        self.0.call(req)
    }
}

pub trait FangsObject {
    fn collect(self, inner: impl FangProc) -> impl FangProc;
}
impl<Fs: Fangs<Inner>> FangsObject for Fs {
    fn collect(self, inner: impl FangProc) -> impl FangProc {
        
    }
}

pub trait Fangs<Inner: FangProc> {
    type Proc: FangProc;
    fn build(self, inner: Inner) -> Self::Proc;
}
const _: () = {
    impl<Inner: FangProc>
    Fangs<Inner> for () {
        type Proc = Inner;
        fn build(self, inner: Inner) -> Self::Proc {
            inner
        }
    }

    impl<Inner: FangProc,
        F1: Fang<Inner>,
    > Fangs<Inner> for (F1,) {
        type Proc = F1::Proc;
        fn build(self, inner: Inner) -> Self::Proc {
            let (f1,) = self;
            f1.chain(inner)
        }
    }

    impl<Inner: FangProc,
        F1: Fang<F2::Proc>,
        F2: Fang<Inner>,
    > Fangs<Inner> for (F1, F2) {
        type Proc = F1::Proc;
        fn build(self, inner: Inner) -> Self::Proc {
            let (f1, f2) = self;
            f1.chain(f2.chain(inner))
        }
    }

    impl<Inner: FangProc,
        F1: Fang<F2::Proc>,
        F2: Fang<F3::Proc>,
        F3: Fang<Inner>,
    > Fangs<Inner> for (F1, F2, F3) {
        type Proc = F1::Proc;
        fn build(self, inner: Inner) -> Self::Proc {
            let (f1, f2, f3) = self;
            f1.chain(f2.chain(f3.chain(inner)))
        }
    }
};

#[cfg(test)] const _: () = {
    trait SetOfFangs<SetOfProcs> {
        fn collect(self, handler: Handler) -> impl FangProc;
    }

    fn __merge<
        A: Fangs<B::Proc>,
        B: Fangs<Inner>,
        Inner: FangProc,
    >(a: A, b: B, inner: Inner) -> impl FangProc {
        a.build(b.build(inner))
    }

    struct Fangs2<
        A: Fangs<B::Proc>,
        B: Fangs<Handler>,
    >(A, B);
    impl<
        A: Fangs<B::Proc>,
        B: Fangs<Handler>,
    > SetOfFangs<(A::Proc, B::Proc)> for Fangs2<A, B> {
        fn collect(self, handler: Handler) -> impl FangProc {
            let Self(a, b) = self;
            a.build(b.build(handler))
        }
    }
    impl<
        A: Fangs<B::Proc>,
        B: Fangs<Handler>,
    > Fangs2<A, B> {
        fn promote<X: Fangs<A::Proc>>(self, x: X) -> Fangs3<X, A, B> {
            let Self(a, b) = self;
            Fangs3(x, a, b)
        }
    }

    struct Fangs3<
        A: Fangs<B::Proc>,
        B: Fangs<C::Proc>,
        C: Fangs<Handler>,
    >(A, B, C);
    impl<
        A: Fangs<B::Proc>,
        B: Fangs<C::Proc>,
        C: Fangs<Handler>,
    > SetOfFangs<(A::Proc, B::Proc, C::Proc)> for Fangs3<A, B, C> {
        fn collect(self, handler: Handler) -> impl FangProc {
            let Self(a, b, c) = self;
            a.build(b.build(c.build(handler)))
        }
    }
};
*/

// pub trait Fangs {
//     fn build(self) -> impl Iterator<Item = Arc<dyn >>;
// }

// pub trait Fangs {
//     fn build(self, handler: Handler) -> impl FangProc;
// }
// const _: () = {
//     impl<
//         F1: Fang<Handler>,
//     > Fangs for (F1,) {
//         fn build(self, handler: Handler) -> impl FangProc {
//             let (f1,) = self;
//             f1.chain(handler)
//         }
//     }
// 
//     impl<
//         F1: Fang<F2::Proc>,
//         F2: Fang<Handler>,
//     > Fangs for (F1, F2) {
//         fn build(self, handler: Handler) -> impl FangProc {
//             let (f1, f2) = self;
//             f1.chain(f2.chain(handler))
//         }
//     }
// 
//     impl<
//         F1: Fang<F2::Proc>,
//         F2: Fang<F3::Proc>,
//         F3: Fang<Handler>,
//     > Fangs for (F1, F2, F3) {
//         fn build(self, handler: Handler) -> impl FangProc {
//             let (f1, f2, f3) = self;
//             f1.chain(f2.chain(f3.chain(handler)))
//         }
//     }
// 
//     impl<
//         F1: Fang<F2::Proc>,
//         F2: Fang<F3::Proc>,
//         F3: Fang<F4::Proc>,
//         F4: Fang<Handler>,
//     > Fangs for (F1, F2, F3, F4) {
//         fn build(self, handler: Handler) -> impl FangProc {
//             let (f1, f2, f3, f4) = self;
//             f1.chain(f2.chain(f3.chain(f4.chain(handler))))
//         }
//     }
// };
