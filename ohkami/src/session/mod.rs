#![cfg(any(feature="rt_tokio",feature="rt_async-std"))]

use std::any::Any;
use std::{pin::Pin, sync::Arc};
use std::panic::{AssertUnwindSafe, catch_unwind};
use crate::__rt__::{TcpStream};
use crate::ohkami::router::RadixRouter;
use crate::{Request, Response};


pub(crate) struct Session {
    router:       Arc<RadixRouter>,
    connection:   TcpStream,
    should_close: bool
}
impl Session {
    pub(crate) fn new(
        router:      Arc<RadixRouter>,
        connection:  TcpStream,
    ) -> Self {
        Self {
            router,
            connection,
            should_close: false,
        }
    }

    pub(crate) async fn manage(mut self) {
        fn panicking(panic: Box<dyn Any + Send>) -> Response {
            if let Some(msg) = panic.downcast_ref::<String>() {
                eprintln!("[Panicked]: {msg}");
            } else if let Some(msg) = panic.downcast_ref::<&str>() {
                eprintln!("[Panicked]: {msg}");
            } else {
                eprintln!("[Panicked]");
            }

            crate::Response::InternalServerError()
        }

        let connection = &mut self.connection;
        while !self.should_close {
            let mut req = Request::init();
            let mut req = unsafe {Pin::new_unchecked(&mut req)};

            let res = match req.as_mut().read(connection).await {
                Some(Ok(())) => {
                    self.should_close = req.headers.Connection() == Some("close");
                    match catch_unwind(AssertUnwindSafe(|| self.router.handle(req.get_mut()))) {
                        Ok(future) => future.await,
                        Err(panic) => panicking(panic),
                    }
                }
                Some(Err(res)) => res,
                None => break
            };

            res.send(connection).await;
        }
    }
}
