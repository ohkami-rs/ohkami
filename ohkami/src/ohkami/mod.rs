#[cfg(test)]
mod _test;

pub(crate) mod routes;
pub use routes::{Route, Routes};

use crate::fang::Fangs;
use crate::router::base::Router;
use std::sync::Arc;

#[cfg(feature="__rt_native__")]
use crate::{__rt__, Session};

/// # Ohkami - a smart wolf who serves your web app
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
///     let api_ohkami = Ohkami::with(Auth, (
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
/// A tuple of types that implement `FromParam` trait e.g. `(&str, usize)`.\
/// If the path contains only one parameter, then you can omit the tuple \
/// e.g. just `param: &str`.\
/// (Current ohkami handles at most *2* path params.)
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
    router: Router,
    /// apply just before merged to another, or just before `howl`ing
    fangs:  Option<Arc<dyn Fangs>>,
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
    pub fn new(routes: impl routes::Routes) -> Self {
        let mut router = Router::new();
        routes.apply(&mut router);
        Self { router, fangs: None, }
    }

    /// Create new `Ohkami` with the fangs that bites requests/responses
    /// came to the `routes`.
    /// 
    /// This always call the fangs when a request came to this `Ohkami`
    /// even if the request path is not registered ( so Ohkami respond
    /// with `404 Not Found` ).
    /// If you'd like to call a fang only around a handler, use *local fangs* like
    /// `"/route".GET((Fang1::new(), Fang2, handler))`.
    /// 
    /// ### args
    /// 
    /// - `fangs`: A tuple of `Fang` items. You can omit tuple when `fangs` contains only one `Fang`.
    /// - `routes`: See [`Ohkami::new`](Ohkami::new)
    /// 
    /// ### example
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
    pub fn with(fangs: impl Fangs + 'static, routes: impl routes::Routes) -> Self {
        let mut router = Router::new();
        routes.apply(&mut router);
        Self { router, fangs: Some(Arc::new(fangs)) }
    }

    pub(crate) fn into_router(self) -> Router {
        let Self { fangs, mut router } = self;

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
    /// - `std::net::ToSocketAddrs` if using `nio` or `glommio`
    /// 
    /// *note* : Keep-Alive timeout is 42 seconds by default.
    /// This is configureable by `OHKAMI_KEEPALIVE_TIMEOUT`
    /// environment variable.
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
        let (router, _) = self.into_router().finalize();
        let router = Arc::new(router);

        let listener = __rt__::bind(address).await;

        let (wg, ctrl_c) = (sync::WaitGroup::new(), sync::CtrlC::new());

        while let Some(accept) = ctrl_c.until_interrupt(listener.accept()).await {
            let (connection, addr) = {
                #[cfg(any(feature="rt_tokio", feature="rt_async-std", feature="rt_smol", feature="rt_nio"))] {
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

        crate::DEBUG!("interrupted, trying graceful shutdown...");
        drop(listener);

        crate::DEBUG!("waiting {} session(s) to finish...", wg.count());
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
                let (router, _) = self.into_router().finalize();
                #[cfg(feature="DEBUG")] ::worker::console_debug!("Done `self.router.finalize`");
                
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

    #[cfg(feature="openapi")]
    #[cfg(feature="__rt_native__")]
    /// Generate OpenAPI document.
    /// 
    /// ### note
    ///  
    /// - Currently, only **JSON** is supported as the document format.
    /// - When the binary size matters, you should prepare a feature flag
    ///   activating `ohkami/openapi` in your package, and put all your codes
    ///   around `openapi` behind that feature via `#[cfg(feature = ...)]` or
    ///   `#[cfg_attr(feature = ...)]`.
    /// - This generates `openapi.json`. Use `generate_to` to configure the
    ///   file path.
    /// 
    /// ### example
    /// 
    /// ```no_run
    /// use ohkami::prelude::*;
    /// use ohkami::openapi::{OpenAPI, Server};
    /// 
    /// // An ordinal Ohkami definition, not special
    /// fn my_ohkami() -> Ohkami {
    ///     Ohkami::new((
    ///         "/hello"
    ///             .GET(|| async {"Hello, OpenAPI!"}),
    ///     ))
    /// }
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let o = my_ohkami();
    /// 
    ///     // Here generating openapi.json
    ///     o.generate(OpenAPI {
    ///         title: "Sample API",
    ///         version: "0.1.9",
    ///         servers: vec![
    ///             Server::at("http://api.example.com/v1")
    ///                 .description("Main (production) server"),
    ///             Server::at("http://staging-api.example.com")
    ///                 .description("Internal staging server for testing")
    ///         ]
    ///      });
    /// 
    ///     o.howl("localhost:5000").await
    /// }
    /// ```
    pub fn generate(&self, metadata: crate::openapi::OpenAPI) {
        self.generate_to("openapi.json", metadata)
    }

    #[cfg(feature="openapi")]
    #[cfg(feature="__rt_native__")]
    pub fn generate_to(&self, file_path: impl AsRef<std::path::Path>, metadata: crate::openapi::OpenAPI) {
        let file_path = file_path.as_ref();
        std::fs::write(file_path, self.__openapi_document_bytes__(metadata))
            .expect(&format!("failed to write OpenAPI document JSON to {}", file_path.display()))
    }

    #[cfg(feature="openapi")]
    #[doc(hidden)]
    pub fn __openapi_document_bytes__(&self, openapi: crate::openapi::OpenAPI) -> Vec<u8> {
        let (router, routes) = (Self {
            router: self.router.clone(),
            fangs:  self.fangs.clone()
        }).into_router().finalize();

        crate::DEBUG!("[openapi_document_bytes] routes = {routes:#?}, router = {router:#?}");

        let doc = router.gen_openapi_doc(routes, openapi);

        let mut bytes = ::serde_json::to_vec_pretty(&doc).expect("failed to serialize OpenAPI document");
        bytes.push(b'\n');

        bytes
    }
}

#[cfg(feature="__rt_native__")]
mod sync {
    pub struct WaitGroup(std::ptr::NonNull<
        std::sync::atomic::AtomicUsize
    >);
    const _: () = {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::ptr::NonNull;
        use std::future::Future;
        use std::task::{Context, Poll};
        use std::pin::Pin;

        unsafe impl Send for WaitGroup {}
        unsafe impl Sync for WaitGroup {}

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

            pub fn done(self) {
                /* just drop */
            }
        }

        impl Drop for WaitGroup {
            #[inline]
            fn drop(&mut self) {
                unsafe {self.0.as_ref()}.fetch_sub(1, Ordering::Release);
            }
        }

        impl Future for WaitGroup {
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if unsafe {self.0.as_ref()}.load(Ordering::Acquire) == 0 {
                    crate::DEBUG!("[WaitGroup::poll] Ready");
                    Poll::Ready(())
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }
    };

    pub struct CtrlC;
    const _: () = {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::future::Future;
        use std::task::{Context, Poll, Waker};
        use std::pin::Pin;

        #[cfg(any(feature="rt_tokio", feature="rt_async-std", feature="rt_smol", feature="rt_nio"))]
        use std::{sync::atomic::AtomicPtr, ptr::null_mut};
        #[cfg(any(feature="rt_glommio"))]
        use std::sync::Mutex;
    
        #[cfg(any(feature="rt_tokio", feature="rt_async-std", feature="rt_smol", feature="rt_nio"))]
        static WAKER: AtomicPtr<Waker> = AtomicPtr::new(null_mut());
        #[cfg(any(feature="rt_glommio"))]
        static WAKER: Mutex<Vec<(usize, Waker)>> = Mutex::new(Vec::new());

        static CATCH: AtomicBool = AtomicBool::new(false);

        impl CtrlC {
            pub fn new() -> Self {
                #[cfg(any(feature="rt_tokio", feature="rt_async-std", feature="rt_smol", feature="rt_nio"))]
                ::ctrlc::set_handler(|| {
                    CATCH.store(true, Ordering::SeqCst);
                    let waker = WAKER.swap(null_mut(), Ordering::SeqCst);
                    if !waker.is_null() {
                        unsafe {Box::from_raw(waker)}.wake();
                    }
                }).expect("Something went wrong with Ctrl-C");

                #[cfg(any(feature="rt_glommio"))]
                ::ctrlc::try_set_handler(|| {
                    CATCH.store(true, Ordering::SeqCst);
                    let lock = &mut *WAKER.lock().unwrap();
                    crate::DEBUG!("Finally {} executors on {} CPU(s)", lock.len(), num_cpus::get());
                    for (_, w) in std::mem::take(lock) {
                        w.wake();
                    }
                }).ok();

                Self
            }

            pub fn until_interrupt<T>(&self, task: impl Future<Output = T>) -> impl Future<Output = Option<T>> {
                return UntilInterrupt(task);

                struct UntilInterrupt<F: Future>(F);
                impl<F: Future> Future for UntilInterrupt<F> {
                    type Output = Option<F::Output>;

                    #[inline]
                    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                        match unsafe {Pin::new_unchecked(&mut self.get_unchecked_mut().0)}.poll(cx) {
                            Poll::Ready(t) => Poll::Ready(Some(t)),
                            Poll::Pending  => if CATCH.load(Ordering::SeqCst) {
                                crate::DEBUG!("[CtrlC::catch] Ready");
                                Poll::Ready(None)
                            } else {
                                #[cfg(any(feature="rt_tokio", feature="rt_async-std", feature="rt_smol", feature="rt_nio"))] {
                                    let prev_waker = WAKER.swap(
                                        Box::into_raw(Box::new(cx.waker().clone())),
                                        Ordering::SeqCst
                                    );
                                    if !prev_waker.is_null() {
                                        unsafe {prev_waker.drop_in_place()}
                                    }
                                }
                                #[cfg(any(feature="rt_glommio"))] {
                                    let current_id = glommio::executor().id();
                                    let current_waker = cx.waker().clone();
                                    let mut lock = WAKER.lock().unwrap();
                                    match lock.iter_mut().find(|(id, _)| (*id == current_id)) {
                                        Some(prev) => *prev = (current_id, current_waker),
                                        None       => lock.push((current_id, current_waker)),
                                    }
                                }
                                Poll::Pending
                            }
                        }
                    }
                }
            }
        }
    };
}

#[cfg(all(debug_assertions, feature="__rt_native__"))]
#[cfg(test)]
#[test] fn can_howl_on_any_native_async_runtime() {
    __rt__::testing::block_on(async {
        crate::util::timeout_in(
            std::time::Duration::from_secs(3),
            Ohkami::new(()).howl(("localhost", __rt__::testing::PORT))
        ).await
    });
}
