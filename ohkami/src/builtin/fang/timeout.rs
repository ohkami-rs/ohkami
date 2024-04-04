use std::time::Duration;


/// # Builtin fang for timeout config
/// 
/// <br>
/// 
/// Set timeout of request handling when a request was handled by that `Ohkami`.
/// 
/// <br>
/// 
/// ---
/// *example.rs*
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::builtin::fang::Timeout;
/// use std::time::Duration;
/// 
/// #[tokio::main]
/// async fn main() {
///     Ohkami::with(Timeout::from_secs(10), (
///         "/hello/:sleep".GET(sleeping_hello),
///     )).howl("0.0.0.0:3000").await
/// }
/// 
/// async fn sleeping_hello(sleep: u64) -> &'static str {
///     tokio::time::sleep(Duration::from_secs(sleep)).await;
///     
///     "Hello, I was sleeping ):"
/// }
/// ```
/// ---
#[derive(Clone, Copy)]
pub struct Timeout(Duration);
impl Timeout {
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }

    pub const fn from_secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }
    pub const fn from_millis(millis: u64) -> Self {
        Self(Duration::from_millis(millis))
    }
    pub const fn from_micros(micros: u64) -> Self {
        Self(Duration::from_micros(micros))
    }

    pub fn from_secs_f32(secs: f32) -> Self {
        Self(Duration::from_secs_f32(secs))
    }
    pub fn from_secs_f64(secs: f64) -> Self {
        Self(Duration::from_secs_f64(secs))
    }
}

const _: () = {
    use std::{future::Future, pin::Pin};
    use std::task::{Context, Poll};
    use crate::{Fang, FangProc, Request, Response, IntoResponse, __rt__::sleep};


    impl<Inner: FangProc> Fang<Inner> for Timeout {
        type Proc = TimeoutProc<Inner>;
        fn chain(&self, inner: Inner) -> Self::Proc {
            TimeoutProc { time: self.0, inner }
        }
    }

    pub struct TimeoutProc<Inner: FangProc> {
        inner: Inner,
        time:  Duration,
    }
    impl<Inner: FangProc> FangProc for TimeoutProc<Inner> {
        fn bite<'b>(&'b self, req: &'b mut Request) -> impl Future<Output = Response> + Send + 'b {
            set_timeout(self.time, self.inner.bite(req))
        }
    }


    /// Based on <https://github.com/tower-rs/tower/blob/master/tower/src/timeout/future.rs>
    pub(super) fn set_timeout<Res: IntoResponse>(
        time:   Duration,
        handle: impl Future<Output = Res>,
    ) -> impl Future<Output = Response> {
        struct Timeout<
            Res: IntoResponse,
            Handle: Future<Output = Res>,
            Sleep:  Future<Output = ()>, // `async-std` doesn't provide the type
        > {
            handle: Handle,
            sleep:  Sleep,
        }

        impl<
            Res: IntoResponse,
            Handle: Future<Output = Res>,
            Sleep:  Future<Output = ()>,
        > Future for Timeout<Res, Handle, Sleep> {
            type Output = Response;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                match unsafe {self.as_mut().map_unchecked_mut(|t| &mut t.handle)}.poll(cx) {
                    Poll::Ready(res) => Poll::Ready(res.into_response()),
                    Poll::Pending    => match unsafe {self.map_unchecked_mut(|t| &mut t.sleep)}.poll(cx) {
                        Poll::Pending  => Poll::Pending,
                        Poll::Ready(_) => Poll::Ready(Response::InternalServerError().text("Timeout")),
                    }
                }
            }
        }

        Timeout { handle, sleep: sleep(time) }
    }
};


#[cfg(all(test, feature="testing", any(feature="rt_tokio",feature="rt_async-std")))]
#[crate::__rt__::test] async fn test_timeout() {
    use crate::prelude::*;
    use crate::testing::*;

    async fn lazy_greeting(
        (name, sleep): (&str, u64)
    ) -> String {
        crate::__rt__::sleep(Duration::from_secs(sleep)).await;

        format!("Hello, {name}!")
    }

    let t = Ohkami::with((
        Timeout::from_secs(2),
    ), (
        "/greet/:name/:sleep".GET(lazy_greeting),
    )).test();


    {
        let req = TestRequest::GET("/greet");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::NotFound);
    }
    {
        let req = TestRequest::PUT("/greet/ohkami/1");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::MethodNotAllowed);
    }
    {
        let req = TestRequest::GET("/greet/ohkami/1");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::OK);
        assert_eq!(res.text(),   Some("Hello, ohkami!"));
    }
    {
        let req = TestRequest::GET("/greet/ohkami/3");
        let res = t.oneshot(req).await;
        assert_eq!(res.status(), Status::InternalServerError);
        assert_eq!(res.text(),   Some("Timeout"));
    }
}
