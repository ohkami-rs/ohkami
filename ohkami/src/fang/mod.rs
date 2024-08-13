#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
mod handler;
#[cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]
pub(crate) use handler::{Handler, IntoHandler};

mod middleware;
pub use middleware::{Fangs, util::FangAction};

mod builtin;
pub use builtin::*;

use crate::{Request, Response};
use std::{future::Future, pin::Pin, ops::Deref};


/// # Core trait for Ohkami's Fang system
/// 
/// See `FangAction` for simple cases!
/// 
/// <br>
/// 
/// ### required
/// - `type Proc` ー associated type implementing `FangProc`, the behavior as a fang
/// - `fn chain` ー how to associate with `inner: Inner: FangProc`
/// 
/// <br>
/// 
/// *impl_example.rs*
/// ```
/// use ohkami::prelude::*;
/// use ohkami::{Fang, FangProc};
/// 
/// struct HelloFang;
/// const _: () = {
///     struct HelloFangProc<I: FangProc> {
///         inner: I
///     }
///     impl<I: FangProc> FangProc for HelloFangProc<I> {
///         async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
///             println!("Hello, fang!");
///             self.inner.bite(req).await
///         }
///     }
/// 
///     impl<I: FangProc> Fang<I> for HelloFang {
///         type Proc = HelloFangProc<I>;
///         fn chain(&self, inner: I) -> Self::Proc {
///             HelloFangProc { inner }
///         }
///     }
/// };
/// 
/// ```
pub trait Fang<Inner: FangProc> {
    type Proc: FangProc;
    fn chain(&self, inner: Inner) -> Self::Proc;
}

pub trait FangProc: Send + Sync + 'static {
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl std::future::Future<Output = Response> + Send;

    /// Default: just `Box::pin(self.bite(req))`.
    /// 
    /// Mainly used for override `bite` when itself returns `Pin<Box<dyn Future>>`.
    #[inline(always)]
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

#[derive(Clone/* copy pointer */)]
pub struct BoxedFPC(&'static (dyn
    FangProcCaller + Send + Sync + 'static
));
const _: () = {
    impl BoxedFPC {
        pub(crate) fn from_proc(proc: impl FangProcCaller + Send + Sync + 'static) -> Self {
            Self(Box::leak(Box::new(proc)))
        }
    }

    impl Deref for BoxedFPC {
        type Target = dyn FangProcCaller + Send + Sync + 'static;
        #[inline(always)]
        fn deref(&self) -> &Self::Target {
            self.0
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
            self.0.call_bite(req)
        }
        #[inline]
        fn bite_boxed<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn Future<Output = Response> + Send + 'b>> {
            self.0.call_bite(req)
        }
    }
};
