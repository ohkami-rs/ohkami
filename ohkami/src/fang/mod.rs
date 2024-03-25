use crate::{handler::Handler, Request, Response};


pub trait Fang<Inner: FangProc> {
    type Proc: FangProc;
    fn chain(self, inner: Inner) -> Self::Proc;
}

pub trait FangProc {
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send + 'b;
}
const _: () = {
    impl FangProc for std::convert::Infallible {
        fn bite<'b>(&'b self, _: &'b mut Request) -> impl std::future::Future<Output = Response> + Send + 'b {
            async {unsafe {std::hint::unreachable_unchecked()}}
        }
    }
    impl FangProc for Handler {
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send + 'b {
            self.handle(req)
        }
    }
};

pub trait Fangs<Proc: FangProc>: SealedFangs<Proc> {}
impl<Proc: FangProc, SF: SealedFangs<Proc>> Fangs<Proc> for SF {}
trait SealedFangs<Proc: FangProc> {
    fn build(self, handler: Handler) -> Proc;
} const _: () = {
    impl SealedFangs<Handler> for () {
        fn build(self, handler: Handler) -> Handler {
            handler
        }
    }

    impl<
        F1:Fang<Handler>,
    > SealedFangs<F1::Proc> for (F1,) {
        fn build(self, handler: Handler) -> F1::Proc {
            let (f1,) = self;
            f1.chain(handler)
        }
    }

    impl<
        F1:Fang<F2::Proc>,
        F2:Fang<Handler>,
    > SealedFangs<F1::Proc> for (F1, F2) {
        fn build(self, handler: Handler) -> F1::Proc {
            let (f1, f2) = self;
            f1.chain(f2.chain(handler))
        }
    }

    impl<
        F1:Fang<F2::Proc>,
        F2:Fang<F3::Proc>,
        F3:Fang<Handler>,
    > SealedFangs<F1::Proc> for (F1, F2, F3) {
        fn build(self, handler: Handler) -> F1::Proc {
            let (f1, f2, f3) = self;
            f1.chain(f2.chain(f3.chain(handler)))
        }
    }
    
    impl<
        F1:Fang<F2::Proc>,
        F2:Fang<F3::Proc>,
        F3:Fang<F4::Proc>,
        F4:Fang<Handler>,
    > SealedFangs<F1::Proc> for (F1, F2, F3, F4) {
        fn build(self, handler: Handler) -> F1::Proc {
            let (f1, f2, f3, f4) = self;
            f1.chain(f2.chain(f3.chain(f4.chain(handler))))
        }
    }
};
