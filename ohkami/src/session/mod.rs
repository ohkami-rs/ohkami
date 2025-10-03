#![cfg(feature = "__rt_native__")]

mod connection;

pub use self::connection::Connection;

use crate::response::Upgrade;
use crate::router::r#final::Router;
use crate::util::with_timeout;
use crate::{Request, Response};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::{any::Any, pin::Pin, sync::Arc, time::Duration};

pub(crate) struct Session {
    connection: Connection,
    router: Arc<Router>,
    ip: std::net::IpAddr,
}

impl Session {
    pub(crate) fn new(
        connection: impl Into<Connection>,
        ip: std::net::IpAddr,
        router: Arc<Router>,
    ) -> Self {
        Self {
            connection: connection.into(),
            ip,
            router,
        }
    }

    pub(crate) async fn manage(mut self) {
        #[cold]
        #[inline(never)]
        fn panicking(panic: Box<dyn Any + Send>) -> Response {
            if let Some(msg) = panic.downcast_ref::<String>() {
                crate::WARNING!("[Panicked]: {msg}");
            } else if let Some(msg) = panic.downcast_ref::<&str>() {
                crate::WARNING!("[Panicked]: {msg}");
            } else {
                crate::WARNING!("[Panicked]");
            }
            crate::Response::InternalServerError()
        }

        #[cold]
        #[inline(never)]
        fn handle_send_failure(error: std::io::Error) {
            use std::io::ErrorKind::*;
            if matches!(
                error.kind(),
                BrokenPipe | ConnectionReset | ConnectionAborted
            ) {
                crate::WARNING!("Client disconnected before response could be sent: {error}");
            } else {
                crate::ERROR!("Failed to send response to the client: {error}");
            }
        }

        let mut req = Request::uninit(self.ip);
        let mut req = Pin::new(&mut req);
        let upgrade = loop {
            req.clear();
            // Apply a fresh timeout for each read, thus resetting the timer on activity.
            match with_timeout(
                Duration::from_secs(crate::CONFIG.keepalive_timeout()),
                req.as_mut().read(&mut self.connection),
            )
            .await
            {
                None => {
                    crate::DEBUG!(
                        "\
                        Reached Keep-Alive timeout. In Ohkami, Keep-Alive timeout \
                        is set to 30 seconds by default and is configurable \
                        by `OHKAMI_KEEPALIVE_TIMEOUT` environment variable.\
                    "
                    );
                    break Upgrade::None;
                }
                Some(read_result) => match read_result {
                    Ok(Some(())) => {
                        let close = matches!(req.headers.connection(), Some("close" | "Close"));

                        let res = match catch_unwind(AssertUnwindSafe({
                            let req = req.as_mut();
                            || self.router.handle(req.get_mut())
                        })) {
                            Ok(future) => future.await,
                            Err(panic) => panicking(panic),
                        };
                        let upgrade = match res.send(&mut self.connection).await {
                            Ok(upgrade) => upgrade,
                            Err(e) => {
                                handle_send_failure(e);
                                break Upgrade::None;
                            }
                        };

                        if !upgrade.is_none() {
                            break upgrade;
                        }
                        if close {
                            break Upgrade::None;
                        }
                    }
                    Ok(None) => break Upgrade::None,
                    Err(mut res) => {
                        res.headers.set().connection("close");
                        if let Err(e) = res.send(&mut self.connection).await {
                            handle_send_failure(e);
                        }
                        break Upgrade::None;
                    }
                },
            }
        };

        match upgrade {
            Upgrade::None => {
                crate::DEBUG!("about to shutdown connection");
            }

            #[cfg(feature = "ws")]
            Upgrade::WebSocket(ws) => {
                crate::DEBUG!("WebSocket session started");

                let aborted = ws
                    .manage_with_timeout(
                        crate::__rt__::sleep(Duration::from_secs(
                            crate::CONFIG.websocket_timeout(),
                        )),
                        self.connection,
                    )
                    .await;
                if aborted {
                    crate::WARNING!(
                        "\
                        WebSocket session aborted by timeout. In Ohkami, \
                        WebSocket timeout is set to 3600 seconds (1 hour) \
                        by default and is configurable by `OHKAMI_WEBSOCKET_TIMEOUT` \
                        environment variable.\
                    "
                    );
                }

                crate::DEBUG!("WebSocket session finished");
            }
        }
    }
}
