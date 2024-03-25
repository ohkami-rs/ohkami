use crate::{handler::Handler, Request, Response};


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

pub trait Fangs<Inner: FangProc> {
    type Proc: FangProc;
    fn build(self, inner: Inner) -> Self::Proc;
}
const _: () = {
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
