//! # Fang - Ohkami's underlying request handling system
//!
//! All handlers and middlewares in Ohkami are built on the `Fang` system:
//!
//! - [`Fang`] ー how to chain a process into an inner `FangProc`
//! - [`FangProc`] ー the process actually *biting* a request and returning a response
//! - [`FangAction`] ー utility trait to implement `Fang` easily
//!
//! Ohkami's built-in fangs are provided in [`middleware`](crate::middleware) module.

mod fangs;
pub use fangs::Fangs;

mod action;
pub use action::FangAction;

pub mod bound;
pub(crate) use bound::*;

/* backward compatibility: built-in fangs and `handler` were exposed here until v0.24 */
#[doc(hidden)]
pub use crate::middleware::{BasicAuth, Context, Cors, Csrf, Enamel, Jwt, JwtToken, enamel};
#[cfg(feature = "__rt_native__")]
#[doc(hidden)]
pub use crate::middleware::Timeout;
#[deprecated(since = "0.25.0", note = "use `ohkami::handler` instead")]
#[doc(hidden)]
pub mod handler {
    pub use crate::handler::*;
}

use crate::{Request, Response};
use std::{future::Future, ops::Deref, pin::Pin};

#[cfg(feature = "openapi")]
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
/// struct HelloFangProc<I: FangProc> {
///     inner: I
/// }
/// impl<I: FangProc> Fang<I> for HelloFang {
///     type Proc = HelloFangProc<I>;
///     fn chain(&self, inner: I) -> Self::Proc {
///         HelloFangProc { inner }
///     }
/// }
/// impl<I: FangProc> FangProc for HelloFangProc<I> {
///     async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
///         println!("Hello, fang!");
///         self.inner.bite(req).await
///     }
/// }
/// ```
pub trait Fang<Inner: FangProc>: SendSyncOnThreaded + 'static {
    type Proc: FangProc;
    fn chain(&self, inner: Inner) -> Self::Proc;

    #[cfg(feature = "openapi")]
    fn openapi_map_operation(&self, operation: openapi::Operation) -> openapi::Operation {
        operation
    }
}

pub trait FangProc: SendSyncOnThreaded + 'static {
    // Here not using `-> impl SendOnThreadedFuture` for
    // rust-analyzer's completion.
    // Currently rust-analyzer can perform completion for `-> impl Future` methods
    // as `async fn ...` **only when** it returns exactly `impl Future (+ something)*`,
    // and he can't do it for `-> impl SendOnThreadedFuture<T>`.

    #[cfg(not(feature = "__rt_threaded__"))]
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl Future<Output = Response>;
    #[cfg(feature = "__rt_threaded__")]
    fn bite<'b>(&'b self, req: &'b mut Request) -> impl Future<Output = Response> + Send;

    /// Mainly used for override `bite` when itself returns `Pin<Box<dyn Future>>`.
    ///
    /// ### default
    /// just `Box::pin(self.bite(req))`
    #[inline(always)]
    fn bite_boxed<'b>(
        &'b self,
        req: &'b mut Request,
    ) -> Pin<Box<dyn SendOnThreadedFuture<Response> + 'b>> {
        Box::pin(self.bite(req))
    }
}

/// `FangProc` but object-safe, returning `Pin<Box<dyn Future>>`.
pub(crate) trait FangProcCaller {
    fn call_bite<'b>(
        &'b self,
        req: &'b mut Request,
    ) -> Pin<Box<dyn SendOnThreadedFuture<Response> + 'b>>;
}
impl<Proc: FangProc> FangProcCaller for Proc {
    #[inline(always)]
    fn call_bite<'b>(
        &'b self,
        req: &'b mut Request,
    ) -> Pin<Box<dyn SendOnThreadedFuture<Response> + 'b>> {
        self.bite_boxed(req)
    }
}

#[derive(Clone /* copy pointer */)]
pub(crate) struct BoxedFPC(&'static (dyn FPCBound + 'static));
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
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl SendOnThreadedFuture<Response> {
            self.0.call_bite(req)
        }

        #[inline]
        fn bite_boxed<'b>(
            &'b self,
            req: &'b mut Request,
        ) -> Pin<Box<dyn SendOnThreadedFuture<Response> + 'b>> {
            self.0.call_bite(req)
        }
    }
};
