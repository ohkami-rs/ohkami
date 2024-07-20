#![cfg(any(feature="rt_tokio",feature="rt_async-std"))]

use std::{any::Any, pin::Pin, sync::Arc};
use std::panic::{AssertUnwindSafe, catch_unwind};
use crate::__rt__::TcpStream;
use crate::response::Upgrade;
use crate::utils::timeout_in;
use crate::ohkami::router::RadixRouter;
use crate::{Request, Response};


pub(crate) struct Session {
    router:     Arc<RadixRouter>,
    connection: TcpStream,
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

        match timeout_in(std::time::Duration::from_secs(crate::env::OHKAMI_KEEPALIVE_TIMEOUT()), async {
            loop {
                let mut req = Request::init();
                let mut req = unsafe {Pin::new_unchecked(&mut req)};
                match req.as_mut().read(&mut self.connection).await {
                    Ok(Some(())) => {
                        let close = matches!(req.headers.Connection(), Some("close" | "Close"));

                        let res = match catch_unwind(AssertUnwindSafe(|| self.router.handle(req.get_mut()))) {
                            Ok(future) => future.await,
                            Err(panic) => panicking(panic),
                        };
                        let upgrade = res.send(&mut self.connection).await;
                        if !upgrade.is_none() {
                            break upgrade
                        }

                        if close {break Upgrade::None}
                    }
                    Ok(None) => break Upgrade::None,
                    Err(res) => {res.send(&mut self.connection).await;},
                };
            }
        }).await {
            Some(Upgrade::None) | None => (),

            #[cfg(all(feature="ws", any(feature="rt_tokio",feature="rt_async-std")))]
            Some(Upgrade::WebSocket((config, handler))) => {
                #[cfg(feature="DEBUG")] {
                    println!("Entered websocket session with TcpStream");
                }

                // SAFETY: `&mut self.connection` is valid while `handle`
                let ws = unsafe {crate::ws::Session::new(&mut self.connection, config)};
                handler(ws).await
            }
        }

        #[cfg(feature="DEBUG")] {
            println!("about to shutdown connection");
        }

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
