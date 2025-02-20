pub mod handler;

mod middleware;
pub use middleware::{Fangs, util::FangAction};

mod builtin;
pub use builtin::*;

pub mod bound;
pub(crate) use bound::*;

use crate::{Request, Response};
use std::{pin::Pin, ops::Deref, future::Future};

#[cfg(feature="openapi")]
use crate::openapi;


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

    #[cfg(feature="openapi")]
    fn openapi_map_operation(&self, operation: openapi::Operation) -> openapi::Operation {
        operation
    }
}

pub trait FangProc: SendSyncOnNative + 'static {
    // Here not using `-> impl SendOnNativeFuture` for
    // rust-analyzer's completion.
    // Currently rust-analyzer can complete `-> Future` methods
    // as `async fn ...` **only when** it returns exactly one of:
    // 
    // * `-> impl Future<Output = T>`
    // * `-> impl Future<Output = T> + Send`
    // * `-> impl Future<Output = T> + Send + 'lifetime`
    // 
    // so `-> impl SendOnNativeFuture<T>` prevents his completion...

    #[cfg(any(feature="rt_worker",))]
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl Future<Output = Response>;
    #[cfg(not(any(feature="rt_worker",)))]
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl Future<Output = Response> + Send;

    /// Mainly used for override `bite` when itself returns `Pin<Box<dyn Future>>`.
    /// 
    /// ### default
    /// just `Box::pin(self.bite(req))`
    #[inline(always)]
    fn bite_boxed<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn SendOnNativeFuture<Response> + 'b>> {
        Box::pin(self.bite(req))
    }
}

/// `FangProc` but object-safe, returning `Pin<Box<dyn Future>>`.
pub(crate) trait FangProcCaller {
    fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn SendOnNativeFuture<Response> + 'b>>;
}
impl<Proc: FangProc> FangProcCaller for Proc {
    #[inline(always)]
    fn call_bite<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn SendOnNativeFuture<Response> + 'b>> {
        self.bite_boxed(req)
    }
}

#[derive(Clone/* copy pointer */)]
pub(crate) struct BoxedFPC(&'static (dyn
    FPCBound + 'static
));
const _: () = {
    impl BoxedFPC {
        pub(crate) fn from_proc(proc: impl FPCBound + 'static) -> Self {
            Self(Box::leak(Box::new(proc)))
        }
    }

    impl Deref for BoxedFPC {
        type Target = dyn FPCBound + 'static;
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
        #[allow(refining_impl_trait)]
        #[inline(always)]
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl SendOnNativeFuture<Response> {
            self.0.call_bite(req)
        }

        #[inline]
        fn bite_boxed<'b>(&'b self, req: &'b mut Request) -> Pin<Box<dyn SendOnNativeFuture<Response> + 'b>> {
            self.0.call_bite(req)
        }
    }
};
