// mod fangs;
// 
// pub use fangs::{FrontFang, BackFang};
// use std::any::TypeId;
// 
// #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
// pub(crate) use fangs::{FrontFangCaller, BackFangCaller};
// 
// #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
// pub use fangs::Fangs;
// 

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

pub trait Fangs<Inner: FangProc>: SealedFangs<Inner> {}
impl<Inner: FangProc, SF: SealedFangs<Inner>> Fangs<Inner> for SF {}
trait SealedFangs<Inner: FangProc> {
    fn build(self, handler: Handler) -> impl FangProc;
} const _: () = {
    impl<Inner: FangProc> SealedFangs<Inner> for () {
        fn build(self, handler: Handler) -> impl FangProc {
            handler
        }
    }
    impl<F1:Fang<Handler>> SealedFangs<F1::Proc> for (F1,) {
        fn build(self, handler: Handler) -> impl FangProc {
            let (f1,) = self;
            f1.chain(handler)
        }
    }
    impl<F1:Fang<Handler>, F2:Fang<F1::Proc>> SealedFangs<F2::Proc> for (F1, F2) {
        fn build(self, handler: Handler) -> impl FangProc {
            let (f1, f2) = self;
            f2.chain(f1.chain(handler))
        }
    }
};


    /*
    
        (L1, L2, L3): Layers: fn(self, handler) -> impl Fang;

        
        (L1, L2, L3) <- H

        ↓

        (L1, L2) (L3 chain H)
                 ------------
                      L3'

        ↓

        (L1) (L2 chain L3')
             --------------
                   L2'

        ↓

        (L1 chain L2')
        --------------
             L1'

        ↓

        L1' :: -> Pin<Box<Future>>
    
    */
