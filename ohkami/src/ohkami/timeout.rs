//! Based on <https://github.com/tower-rs/tower/blob/master/tower/src/timeout/future.rs>

use std::task::{Context, Poll};
use std::{future::Future, pin::Pin};
use crate::__rt__::time::{Sleep, sleep};
use crate::Response;


pub(super) fn set_timeout(
    time:   crate::builtin::Timeout,
    handle: impl Future<Output = Response>,
) -> impl Future<Output = Response> {
    struct Timeout<H: Future<Output = Response>> {
        handle: H,
        time:   Sleep,
    }

    impl<H: Future<Output = Response>> Future for Timeout<H> {
        type Output = Response;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            match unsafe {self.as_mut().map_unchecked_mut(|t| &mut t.handle)}.poll(cx) {
                Poll::Ready(res) => Poll::Ready(res),
                Poll::Pending    => match unsafe {self.map_unchecked_mut(|t| &mut t.time)}.poll(cx) {
                    Poll::Pending  => Poll::Pending,
                    Poll::Ready(_) => Poll::Ready(Response::InternalServerError().text("Timeout")),
                }
            }
        }
    }

    Timeout { handle, time:sleep(time.0) }
}
