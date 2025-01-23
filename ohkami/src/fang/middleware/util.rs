use super::super::{Fang, FangProc};
use super::super::bound::{SendSyncOnNative, SendOnNativeFuture};
use crate::{Request, Response};

#[cfg(feature="openapi")]
use crate::openapi;


/// # Fang action - utility wrapper of `Fang`
/// 
/// `FangAction` provides 2 actions:
/// 
/// - `fore` ... *bite* a `&mut Request`, maybe early returning `Err(Response)`, before a handler is called
/// - `back` ... *bite* a `&mut Response` after a handler is called
/// 
/// Both of them perform nothing by default.
/// 
/// <br>
/// 
/// `T: FangAction` automatically implements `Fang` that performs as
/// 
/// ```
/// # use ohkami::{prelude::*, Fang, FangProc};
/// # #[derive(Clone)]
/// # struct DummyProc<
/// #     A: FangAction + Clone,
/// #     I: FangProc + Clone,
/// # > {
/// #     action: A,
/// #     inner:  I,
/// # }
/// # impl<
/// #     A: FangAction + Clone,
/// #     I: FangProc + Clone,
/// # > FangProc for DummyProc<A, I> {
/// async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
///     let Self { action, inner } = self;
///     match action.fore(req).await {
///         Err(e) => e,
///         Ok(()) => {
///             let mut res = inner.bite(req).await;
///             action.back(&mut res).await;
///             res
///         }
///     }
/// }
/// # }
/// ```
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```
/// use ohkami::prelude::*;
/// 
/// #[derive(Clone)]
/// struct SimpleLogger;
/// impl FangAction for SimpleLogger {
///     async fn fore<'a>(&'a self, req: &'a mut Request) -> Result<(), Response> {
///         println!("[request] {req:?}");
///         Ok(())
///     }
///     async fn back<'a>(&'a self, res: &'a mut Response) {
///         println!("[response] {res:?}");
///     }
/// }
/// ```
pub trait FangAction: Clone + SendSyncOnNative + 'static {
    /// *fore fang*, that bites a request before a handler.
    /// 
    /// ### default
    /// just return `Ok(())`
    #[allow(unused_variables)]
    fn fore<'a>(&'a self, req: &'a mut Request) -> impl SendOnNativeFuture<Result<(), Response>> {
        async {Ok(())}
    }

    /// *back fang*, that bites a response after a handler.
    /// 
    /// ### default
    /// just return `()`
    #[allow(unused_variables)]
    fn back<'a>(&'a self, res: &'a mut Response) -> impl SendOnNativeFuture<()> {
        async {}
    }

    #[cfg(feature="openapi")]
    fn openapi_map_operation(operation: openapi::Operation) -> openapi::Operation {
        operation
    }
}
const _: () = {
    impl<A: FangAction, I: FangProc> Fang<I> for A {
        type Proc = FangActionProc<A, I>;
        fn chain(&self, inner: I) -> Self::Proc {
            FangActionProc {
                action: self.clone(),
                inner
            }
        }

        #[cfg(feature="openapi")]
        fn openapi_map_operation(&self, operation: openapi::Operation) -> openapi::Operation {
            <Self as FangAction>::openapi_map_operation(operation)
        }
    }

    pub struct FangActionProc<A: FangAction, I: FangProc> {
        action: A,
        inner:  I,
    }
    impl<A: FangAction, I: FangProc> FangProc for FangActionProc<A, I> {
        #[inline(always)]
        async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
            let Self { action, inner } = self;
            match action.fore(req).await {
                Err(e) => e,
                Ok(()) => {
                    let mut res = inner.bite(req).await;
                    action.back(&mut res).await;
                    res
                }
            }
        }
    }
};




#[cfg(all(test, debug_assertions, feature="__rt_native__", feature="DEBUG"))]
mod test {
    use super::*;
    use crate::prelude::*;
    use crate::testing::*;

    #[test]
    fn availablity() {
        use std::sync::{Mutex, OnceLock};

        fn messages() -> &'static Mutex<Vec<String>> {
            static MESSAGES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
            MESSAGES.get_or_init(|| Mutex::new(Vec::new()))
        }

        #[derive(Clone)]
        struct GreetingFang { name: &'static str }
        const _: () = {
            impl<I: FangProc> Fang<I> for GreetingFang {
                type Proc = GreetingFangProc<I>;
                fn chain(&self, inner: I) -> Self::Proc {
                    GreetingFangProc { fang: self.clone(), inner }
                }
            }

            struct GreetingFangProc<I: FangProc> {
                fang:  GreetingFang,
                inner: I
            }
            impl<I: FangProc> FangProc for GreetingFangProc<I> {
                async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
                    {
                        messages().lock().unwrap().push(format!("Hello, {}!", self.fang.name));
                    }
                    let res = self.inner.bite(req).await;
                    {
                        messages().lock().unwrap().push(format!("Bye, {}!", self.fang.name));
                    }
                    res
                }
            }
        };

        #[derive(Clone)]
        struct GreetingFangWithAction { name: &'static str }
        impl FangAction for GreetingFangWithAction {
            async fn fore<'b>(&'b self, _req: &'b mut Request) -> Result<(), Response> {
                messages().lock().unwrap().push(format!("Hello, {}!", self.name));
                Ok(())
            }
            async fn back<'b>(&'b self, _res: &'b mut Response) {
                messages().lock().unwrap().push(format!("Bye, {}!", self.name));
            }
        }

        let t = Ohkami::new((
            GreetingFang { name: "Clerk" },
            GreetingFangWithAction { name: "John" },
            "/greet"
                .POST(|| async {"Hi, I'm Handler!"}),
        )).test();

        crate::__rt__::testing::block_on(async {
            {
                let req = TestRequest::POST("/greet");
                let res = t.oneshot(req).await;

                assert_eq!(res.status(), Status::OK);
                assert_eq!(&*messages().lock().unwrap(), &[
                    "Hello, Clerk!",
                    "Hello, John!",
                    "Bye, John!",
                    "Bye, Clerk!",
                ]);
            }
        });
    }
}
