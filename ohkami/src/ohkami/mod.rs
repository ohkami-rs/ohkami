#![cfg(feature="__rt__")]

#[cfg(test)]
mod _test;

pub(crate) mod build;

pub use build::{Route, Routes};

use crate::fang::Fangs;
use crate::router::base::Router;
use std::sync::Arc;

#[cfg(feature="__rt_native__")]
use crate::{__rt__, Session};

/// # Ohkami - a robust wolf who serves your web app
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// # use ohkami::prelude::*;
/// # use ohkami::serde::Serialize;
/// # use ohkami::typed::status::{OK, Created};
/// # use ohkami::format::JSON;
/// # use ohkami::{Fang, FangProc};
/// 
/// struct Auth;
/// impl<I: FangProc> Fang<I> for Auth {
///     /* 〜 */
/// #   type Proc = AuthProc;
/// #   fn chain(&self, inner: I) -> Self::Proc {
/// #       AuthProc
/// #   }
/// # }
/// # struct AuthProc;
/// # impl FangProc for AuthProc {
/// #     async fn bite<'b>(&'b self, req: &'b mut Request) -> Response {
/// #         Response::NotImplemented()
/// #     }
/// # }
/// 
/// # #[derive(Serialize)]
/// # struct User {
/// #     id:   usize,
/// #     name: String,
/// #     age:  Option<usize>,
/// # }
/// # 
/// # enum APIError {
/// #     UserNotFound
/// # }
/// # impl IntoResponse for APIError {
/// #     fn into_response(self) -> Response {
/// #         match self {
/// #             Self::UserNotFound => Response::InternalServerError()
/// #         }
/// #     }
/// # }
/// # 
/// # async fn health_check() -> impl IntoResponse {
/// #     Status::NoContent
/// # }
/// # 
/// # async fn create_user() -> Created<JSON<User>> {
/// #     Created(JSON(User {
/// #         id:   42,
/// #         name: String::from("ohkami"),
/// #         age:  None,
/// #     }))
/// # }
/// # 
/// # async fn get_user_by_id(id: usize) -> Result<JSON<User>, APIError> {
/// #     Ok(JSON(User {
/// #         id,
/// #         name: String::from("ohkami"),
/// #         age:  Some(2),
/// #     }))
/// # }
/// # 
/// # async fn update_user(id: usize) -> impl IntoResponse {
/// #     Status::OK
/// # }
/// 
/// fn my_ohkami() -> Ohkami {
///     let api_ohkami = Ohkami::with((Auth,), (
///         "/users"
///             .POST(create_user),
///         "/users/:id"
///             .GET(get_user_by_id)
///             .PATCH(update_user),
///     ));
/// 
///     Ohkami::new((
///         "/hc" .GET(health_check),
///         "/api".By(api_ohkami),
///     ))
/// }
/// ```
/// 
/// <br>
/// 
/// #### handler schema：
/// `async ({path_params}?, {FromRequest type}s...) -> {IntoResponse type}`
/// 
/// #### path_params：
/// A tuple of types that implement `FromParam` trait.\
/// If the path contains only one parameter, then you can omit the tuple.\
/// (In current ohkami, at most *2* path params can be handled.)
/// 
/// <br>
/// 
/// ```
/// use ohkami::prelude::*;
/// 
/// struct MyParam;
/// impl<'p> ohkami::FromParam<'p> for MyParam {
///     type Error = std::convert::Infallible;
///     fn from_param(param: std::borrow::Cow<'p, str>) -> Result<Self, Self::Error> {
///         Ok(MyParam)
///     }
/// }
/// 
/// async fn handler_1(param: (MyParam,)) -> Response {
///     todo!()
/// }
/// 
/// async fn handler_2(str_param: &str) -> Response {
///     todo!()
/// }
/// ```
pub struct Ohkami {
    pub(crate) routes: Router,

    /// apply just before merged to another or called `howl`
    pub(crate) fangs:  Option<Arc<dyn Fangs>>,
}


impl Ohkami {
    /// Create new `Ohkami` on the routing.
    /// 
    /// ---
    ///
    /// `routes` is a routing item or a tuple of them :
    /// 
    /// ```
    /// # use ohkami::Route;
    /// #
    /// # async fn handler1() -> &'static str {"1"}
    /// # async fn handler2() -> &'static str {"2"}
    /// # async fn handler3() -> &'static str {"3"}
    /// #
    /// # let _ =
    /// (
    ///     "/a"
    ///         .GET(handler1)
    ///         .POST(handler2),
    ///     "/b"
    ///         .PUT(handler3),
    ///     //...
    /// )
    /// # ;
    /// ```
    /// 
    /// ---
    /// 
    /// Handler is an _**async**_ function :
    /// 
    /// > `({path params}, {FromRequest values},...) -> {IntoResponse value}`
    ///
    /// `{path params}` is a `FromParam` value or a tuple of them
    pub fn new(routes: impl build::Routes) -> Self {
        let mut router = Router::new();
        routes.apply(&mut router);

        Self {
            routes: router,
            fangs:  None,
        }
    }

    /// Create new ohkami with the fangs on the routing.
    /// 
    /// ---
    ///
    /// `fangs: impl Fangs` is an tuple of `Fang` items.
    /// 
    /// **NOTE**: You can omit tuple when `fangs` contains only one `Fang`.
    /// 
    /// <br>
    /// 
    /// ---
    /// 
    /// ```
    /// use ohkami::prelude::*;
    /// 
    /// #[derive(Clone)]
    /// struct AuthFang;
    /// impl FangAction for AuthFang {
    ///     //...
    /// }
    /// 
    /// # async fn handler1() -> &'static str {"1"}
    /// # async fn handler2() -> &'static str {"2"}
    /// # async fn handler3() -> &'static str {"3"}
    /// #
    /// # let _ =
    /// Ohkami::with(AuthFang, (
    ///     "/a"
    ///         .GET(handler1)
    ///         .POST(handler2),
    ///     "/b"
    ///         .PUT(handler3),
    ///     //...
    /// ))
    /// # ;
    /// ```
    pub fn with(fangs: impl Fangs + 'static, routes: impl build::Routes) -> Self {
        let mut router = Router::new();
        routes.apply(&mut router);

        Self {
            routes: router,
            fangs:  Some(Arc::new(fangs)),
        }
    }

    pub(crate) fn into_router(self) -> Router {
        let Self { routes: mut router, fangs } = self;

        if let Some(fangs) = fangs {
            router.apply_fangs(router.id(), fangs);
        }

        #[cfg(feature="DEBUG")]
        println!("{router:#?}");

        router
    }

    #[cfg(feature="__rt_native__")]
    /// Start serving at `address`!
    /// 
    /// `address` is：
    /// 
    /// - `tokio::net::ToSocketAddrs` if using `tokio`
    /// - `async_std::net::ToSocketAddrs` if using `async-std`
    /// - `smol::net::AsyncToSocketAddrs` if using `smol`
    /// - `std::net::ToSocketAddrs` if using `glommio`
    /// 
    /// *note* : Keep-Alive timeout is 42 seconds and this is not
    /// configureable by user (it'll be in future version...)
    /// 
    /// <br>
    /// 
    /// ---
    /// 
    /// *example.rs*
    /// ```no_run
    /// use ohkami::prelude::*;
    /// use ohkami::typed::status;
    /// 
    /// async fn hello() -> &'static str {
    ///     "Hello, ohkami!"
    /// }
    /// 
    /// async fn health_check() -> status::NoContent {
    ///     status::NoContent
    /// }
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     Ohkami::new((
    ///         "/".GET(hello),
    ///         "/healthz".GET(health_check),
    ///     )).howl("localhost:5000").await
    /// }
    /// ```
    /// 
    /// ---
    /// 
    /// *example_glommio.rs*
    /// ```ignore
    /// use ohkami::prelude::*;
    /// use ohkami::util::num_cpus;
    /// use glommio::{LocalExecutorPoolBuilder, PoolPlacement, CpuSet};
    /// 
    /// async fn hello() -> &'static str {
    ///     "Hello, ohkami!"
    /// }
    /// 
    /// fn main() {
    ///     LocalExecutorPoolBuilder::new(PoolPlacement::MaxSpread(
    ///         num_cpus::get(), CpuSet::online().ok()
    ///     )).on_all_shards(|| {
    ///         Ohkami::new((
    ///             "/user/:id"
    ///                 .GET(echo_id),
    ///         )).howl("0.0.0.0:3000")
    ///     }).unwrap().join_all();
    /// }
    /// ```
    pub async fn howl(self, address: impl __rt__::ToSocketAddrs) {
        let router = Arc::new(self.into_router().finalize());

        let listener = __rt__::bind(address).await;

        let (wg, inturrupt) = (sync::WaitGroup::new(), sync::CtrlC::new());

        loop {
            let (accept, inturrupt) = (listener.accept(), inturrupt.inturrupted());

            #[cfg(any(feature="rt_async-std", feature="rt_smol", feature="rt_glommio"))]
            let (accept, inturrupt) = {use ::futures_util::FutureExt;
                (accept.fuse(), inturrupt.fuse())
            };

            __rt__::select! {
                accept = accept => {
                    let (connection, addr) = {
                        #[cfg(any(feature="rt_tokio", feature="rt_async-std", feature="rt_smol"))] {
                            let Ok((connection, addr)) = accept else {continue};
                            (connection, addr)
                        }
                        #[cfg(any(feature="rt_glommio"))] {
                            let Ok(connection) = accept else {continue};
                            let Ok(addr) = connection.peer_addr() else {continue};
                            (connection, addr)
                        }
                    };

                    let session = Session::new(
                        router.clone(),
                        connection,
                        addr.ip()
                    );

                    let wg = wg.add();
                    __rt__::spawn(async move {
                        session.manage().await;
                        wg.done();
                    });
                }
                _ = inturrupt => {
                    crate::DEBUG!("Recieved Ctrl-C, trying graceful shutdown...");
                    drop(listener);
                    break
                }
            }
        }

        crate::DEBUG!("Waiting {} session(s) to finish...", wg.count());
        wg.await;
    }

    #[cfg(feature="rt_worker")]
    #[doc(hidden)]
    pub async fn __worker__(self,
        req: ::worker::Request,
        env: ::worker::Env,
        ctx: ::worker::Context,
    ) -> ::worker::Response {
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Called `#[ohkami::worker]`; req: {req:?}");

        let mut ohkami_req = crate::Request::init();
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `ohkami::Request::init`");

        let mut ohkami_req = unsafe {std::pin::Pin::new_unchecked(&mut ohkami_req)};
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Put request in `Pin`");

        let take_over = ohkami_req.as_mut().take_over(req, env, ctx).await;
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `ohkami::Request::take_over`: {ohkami_req:?}");

        let ohkami_res = match take_over {
            Ok(()) => {#[cfg(feature="DEBUG")] ::worker::console_debug!("`take_over` succeed");

                let router = self.into_router();
                #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `Ohkami::into_router`");

                let router = router.finalize();
                #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `Router::finalize` (without compressions)");
                
                let mut res = router.handle(&mut ohkami_req).await;
                res.complete();
                res
            }
            Err(e) => {#[cfg(feature="DEBUG")] ::worker::console_debug!("`take_over` returned an error response: {e:?}");
                e
            }
        };
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Successfully generated ohkami::Response: {ohkami_res:?}");

        let res = ohkami_res.into();
        #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `ohkami::Response` --into--> `worker::Response`: {res:?}");

        res
    }
}

#[cfg(feature="__rt_native__")]
mod sync {
    use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
    use std::ptr::NonNull;

    use std::future::Future;
    use std::task::{Context, Poll};
    use std::pin::Pin;

    pub struct WaitGroup(
        NonNull<AtomicUsize>
    );
    const _: () = {
        unsafe impl Send for WaitGroup {}
        unsafe impl Sync for WaitGroup {}

        impl Future for WaitGroup {
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if unsafe {self.0.as_ref()}.load(Ordering::Acquire) == 0 {
                    Poll::Ready(())
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }

        impl WaitGroup {
            pub fn new() -> Self {
                let n = AtomicUsize::new(0);
                let n = Box::leak(Box::new(n));
                Self(NonNull::new(n).unwrap())
            }

            #[cfg(feature="DEBUG")]
            pub fn count(&self) -> usize {
                unsafe {self.0.as_ref()}.load(Ordering::Relaxed)
            }

            #[inline]
            pub fn add(&self) -> Self {
                let ptr = self.0;
                unsafe {ptr.as_ref()}.fetch_add(1, Ordering::Relaxed);
                Self(ptr)
            }

            #[inline]
            pub fn done(&self) {
                unsafe {self.0.as_ref()}.fetch_sub(1, Ordering::Release);
            }
        }
    };

    pub struct CtrlC;
    const _: () = {
        static INTURRUPTED: AtomicBool = AtomicBool::new(false);

        impl Copy for CtrlC {}
        impl Clone for CtrlC {fn clone(&self) -> Self {Self}}

        impl CtrlC {
            pub fn new() -> Self {
                ::ctrlc::set_handler(|| INTURRUPTED.store(true, Ordering::Release))
                    .expect("Something went wrong around Ctrl-C");
                Self
            }

            pub const fn inturrupted(&self) -> impl Future<Output = ()> + 'static {
                struct Inturrupt;
                impl Future for Inturrupt {
                    type Output = ();
                    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                        if INTURRUPTED.load(Ordering::Acquire) {
                            Poll::Ready(())
                        } else {
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                    }
                }

                Inturrupt
            }
        }
    };
}

#[cfg(feature="testing")]
#[cfg(test)]
#[test] fn can_howl() {
    __rt__::block_on(
        crate::util::timeout_in(
            std::time::Duration::from_secs(3),
            Ohkami::new(()).howl("localhost:3000")
        )
    );
}
