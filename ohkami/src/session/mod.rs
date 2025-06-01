#![cfg(feature="__rt_native__")]

use std::{any::Any, pin::Pin, sync::Arc, time::Duration};
use std::panic::{AssertUnwindSafe, catch_unwind};
use crate::__rt__::TcpStream;
use crate::response::Upgrade;
use crate::util::timeout_in;
use crate::router::r#final::Router;
use crate::{Request, Response};

pub(crate) struct Session {
    router:     Arc<Router>,
    connection: TcpStream,
    ip:         std::net::IpAddr,
}

impl Session {
    pub(crate) fn new(
        router:     Arc<Router>,
        connection: TcpStream,
        ip:         std::net::IpAddr
    ) -> Self {
        Self {
            router,
            connection,
            ip
        }
    }

    pub(crate) async fn manage(mut self) {
        #[cold]
        #[inline(never)]
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

        #[cold]
        #[inline(never)]
        fn handle_send_failure(error: std::io::Error) {
            use std::io::ErrorKind::*;
            if matches!(error.kind(), BrokenPipe | ConnectionReset | ConnectionAborted) {
                crate::warning!("[WARNING] Client disconnected before response could be sent: {error}");
            } else {
                crate::warning!("[ERROR] Failed to send response to the client: {error}");
            }
        }

        match timeout_in(Duration::from_secs(crate::CONFIG.keepalive_timeout()), async {
            let mut req = Request::init(self.ip);
            let mut req = unsafe {Pin::new_unchecked(&mut req)};
            loop {
                req.clear();
                match req.as_mut().read(&mut self.connection).await {
                    Ok(Some(())) => {
                        let close = matches!(req.headers.Connection(), Some("close" | "Close"));

                        let res = match catch_unwind(AssertUnwindSafe({
                            let req = req.as_mut();
                            || self.router.handle(req.get_mut())
                        })) {
                            Ok(future) => future.await,
                            Err(panic) => panicking(panic),
                        };
                        let upgrade = res.send(&mut self.connection).await?;

                        if !upgrade.is_none() {break Ok(upgrade);}
                        if close {break Ok(Upgrade::None);}
                    }
                    Ok(None) => {break Ok(Upgrade::None);}
                    Err(res) => {res.send(&mut self.connection).await?;}
                }
            }
        }).await {
            None => crate::warning!("[WARNING] \
                Session timeouted. In Ohkami, Keep-Alive timeout \
                is set to 30 seconds by default and is configurable \
                by `OHKAMI_KEEPALIVE_TIMEOUT` environment variable.\
            "),

            Some(Err(e)) => handle_send_failure(e),

            Some(Ok(Upgrade::None)) => crate::DEBUG!("about to shutdown connection"),

            #[cfg(feature="ws")]
            Some(Ok(Upgrade::WebSocket(ws))) => {
                crate::DEBUG!("WebSocket session started");

                let aborted = ws.manage_with_timeout(
                    Duration::from_secs(crate::CONFIG.websocket_timeout()),
                    self.connection
                ).await;
                if aborted {
                    crate::warning!("[WARNING] \
                        WebSocket session aborted by timeout. In Ohkami, \
                        WebSocket timeout is set to 3600 seconds (1 hour) \
                        by default and is configurable by `OHKAMI_WEBSOCKET_TIMEOUT` \
                        environment variable.\
                    ");
                }

                crate::DEBUG!("WebSocket session finished");
            }
        }
    }
}
