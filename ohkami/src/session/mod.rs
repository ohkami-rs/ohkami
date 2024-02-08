use std::any::Any;
use std::{pin::Pin, sync::Arc};
use std::panic::{AssertUnwindSafe, catch_unwind};
use crate::__rt__::{TcpStream};
use crate::ohkami::router::RadixRouter;
use crate::{Request, Response};


pub(crate) struct Session {
    router:     Arc<RadixRouter>,
    connection: TcpStream,
}
impl Session {
    pub(crate) fn new(
        router:     Arc<RadixRouter>,
        connection: TcpStream,
    ) -> Self {
        Self {
            router,
            connection,
        }
    }

    pub(crate) async fn manage(mut self) {
        #[cold] fn panicking(panic: Box<dyn Any + Send>) -> Response {
            if let Some(msg) = panic.downcast_ref::<String>() {
                eprintln!("Panicked: {msg}");
            } else if let Some(msg) = panic.downcast_ref::<&str>() {
                eprintln!("Panicked: {msg}");
            } else {
                eprintln!("Panicked");
            }

            crate::Response::InternalServerError()
        }

        const LOOP_LIMIT: u8 = 16;

        let connection = &mut self.connection;

        for _ in 0..LOOP_LIMIT {
            let mut req = Request::init();
            let mut req = unsafe {Pin::new_unchecked(&mut req)};
            if req.as_mut().read(connection).await.is_none() {break}

            let close = req.headers.Connection().is_some_and(|c| c == "close");

            let res = match catch_unwind(AssertUnwindSafe(|| self.router.handle(req.get_mut()))) {
                Ok(future) => future.await,
                Err(panic) => panicking(panic),
            };
            res.send(connection).await;

            if close {break}
        }
    }
}
