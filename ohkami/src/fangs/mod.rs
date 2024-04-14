#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
mod handler;
#[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
pub(crate) use handler::{Handler, IntoHandler};

mod middleware;
pub use middleware::{Fangs, util};

use crate::{Request, Response};
use std::{future::Future, pin::Pin, ops::Deref};


pub trait Fang<Inner: FangProc> {
    type Proc: FangProc;
    fn chain(&self, inner: Inner) -> Self::Proc;
}

pub trait FangProc: Send + Sync + 'static {
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send;

    /// Default: `Box::pin(self.bite(req))`.
    /// 
    /// Override when `bite` itself returns `Pin<Box<dyn Future>>`.
    fn bite_boxed<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
        Box::pin(self.bite(req))
    }
}

/// `FangProc` but object-safe, returning `Pin<Box<dyn Future>>`.
pub trait FangProcCaller {
    fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>>;
}
impl<Proc: FangProc> FangProcCaller for Proc {
    #[inline(always)]
    fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
        self.bite_boxed(req)
    }
}

pub struct BoxedFPC(Box<dyn
    FangProcCaller + Send + Sync + 'static
>);
impl BoxedFPC {
    pub(crate) fn from_proc(proc: impl FangProcCaller + Send + Sync + 'static) -> Self {
        Self(Box::new(proc))
    }
}
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

    impl FangProc for BoxedFPC {
        #[inline(always)]
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send {
            (&*self.0).call_bite(req)
        }

        #[inline]
        fn bite_boxed<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
            (&*self.0).call_bite(req)
        }
    }
};
