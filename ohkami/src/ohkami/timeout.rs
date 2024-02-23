//! Based on <https://github.com/tower-rs/tower/blob/master/tower/src/timeout/future.rs>

use std::task::{Context, Poll};
use std::{future::Future, pin::Pin};
use crate::__rt__::sleep;
use crate::Response;


pub(super) fn set_timeout(
    time:   crate::builtin::Timeout,
    handle: impl Future<Output = Response>,
) -> impl Future<Output = Response> {
    struct Timeout<
        Handle: Future<Output = Response>,
        Sleep:  Future<Output = ()>,
    > {
        handle: Handle,
        sleep:   Sleep,
    }

    impl<
        Handle: Future<Output = Response>,
        Sleep:  Future<Output = ()>,
    > Future for Timeout<Handle, Sleep> {
        type Output = Response;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            match unsafe {self.as_mut().map_unchecked_mut(|t| &mut t.handle)}.poll(cx) {
                Poll::Ready(res) => Poll::Ready(res),
                Poll::Pending    => match unsafe {self.map_unchecked_mut(|t| &mut t.sleep)}.poll(cx) {
                    Poll::Pending  => Poll::Pending,
                    Poll::Ready(_) => Poll::Ready(Response::InternalServerError().text("Timeout")),
                }
            }
        }
    }

    Timeout { handle, sleep:sleep(time.0) }
}
