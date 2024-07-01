#![cfg(any(feature="rt_tokio",feature="rt_async-std"))]

use std::{any::Any, pin::Pin, sync::Arc, future::Future, time::Duration, task::Poll};
use std::panic::{AssertUnwindSafe, catch_unwind};
use crate::__rt__::{TcpStream, sleep};
use crate::ohkami::router::RadixRouter;
use crate::{Request, Response};


pub(crate) struct Session {
    router:       Arc<RadixRouter>,
    connection:   TcpStream,
}
impl Session {
    pub(crate) fn new(
        router:      Arc<RadixRouter>,
        connection:  TcpStream,
    ) -> Self {
        Self {
            router,
            connection,
        }
    }

    pub(crate) async fn manage(mut self) {
        #[cold] #[inline(never)]
        fn panicking(panic: Box<dyn Any + Send>) -> Response {
            if let Some(msg) = panic.downcast_ref::<String>() {
                crate::warning!("[Panicked]: {msg}");
            } else if let Some(msg) = panic.downcast_ref::<&str>() {
                crate::warning!("[Panicked]: {msg}");
            } else {
                crate::warning!("[Panicked]");
            }
            crate::Response::InternalServerError()
        }

        fn timeout_in(
            secs: u64,
            proc: impl Future<Output = ()>,
        ) -> impl Future<Output = ()> {
            struct Timeout<Sleep, Proc> { sleep: Sleep, proc: Proc }
            impl<Sleep: Future<Output = ()>, Proc: Future<Output = ()>> Future for Timeout<Sleep, Proc> {
                type Output = ();
                
                #[inline]
                fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
                    unsafe {
                        match self.as_mut().map_unchecked_mut(|t| &mut t.proc).poll(cx) {
                            Poll::Ready(()) => Poll::Ready(()),
                            Poll::Pending   => self.map_unchecked_mut(|t| &mut t.sleep).poll(cx),
                        }
                    }
                }
            }

            Timeout { sleep: sleep(Duration::from_secs(secs)), proc }
        }

        /* async-std doesn't provide split */
        #[cfg(feature="rt_tokio")]
        let (mut r, mut w) = self.connection.split();
        #[cfg(feature="rt_async-std")]
        let c = &mut self.connection;

        #[cfg(feature="rt_tokio")]
        macro_rules! read {($req:ident) => {$req.as_mut().read(&mut r)};}
        #[cfg(feature="rt_async-std")]
        macro_rules! read {($req:ident) => {$req.as_mut().read(c)};}

        #[cfg(feature="rt_tokio")]
        macro_rules! send {($res:ident) => {$res.send(&mut w)};}
        #[cfg(feature="rt_async-std")]
        macro_rules! send {($res:ident) => {$res.send(c)};}

        timeout_in(42/* TODO: make this configurable by user */, async {
            loop {
                let mut req = Request::init();
                let mut req = unsafe {Pin::new_unchecked(&mut req)};
                match read!(req).await {
                    Ok(Some(())) => {
                        let close = matches!(req.headers.Connection(), Some("close" | "Close"));
                        let res = match catch_unwind(AssertUnwindSafe(|| self.router.handle(req.get_mut()))) {
                            Ok(future) => future.await,
                            Err(panic) => panicking(panic),
                        };
                        send!(res).await;
                        if close {break}
                    }
                    Ok(None) => break,
                    Err(res) => send!(res).await,
                };
            }
        }).await;

        if let Some(err) = {
            #[cfg(feature="rt_tokio")] {use crate::__rt__::AsyncWriter;
                self.connection.shutdown().await
            }
            #[cfg(feature="rt_async-std")] {
                self.connection.shutdown(std::net::Shutdown::Both)
            }
        }.err() {
            match err.kind() {
                std::io::ErrorKind::NotConnected => (),
                _ => panic!("Failed to shutdown stream: {err}")
            }
        }
    }
}
