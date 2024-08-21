#![cfg(any(feature="rt_tokio",feature="rt_async-std"))]

use std::{any::Any, pin::Pin, sync::Arc, time::Duration};
use std::panic::{AssertUnwindSafe, catch_unwind};
use crate::__rt__::TcpStream;
use crate::response::Upgrade;
use crate::utils::timeout_in;
use crate::ohkami::router::RadixRouter;
use crate::{Request, Response};


mod env {
    #![allow(unused, non_snake_case)]

    use std::sync::OnceLock;

    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub(crate) fn OHKAMI_KEEPALIVE_TIMEOUT() -> u64 {
        static OHKAMI_KEEPALIVE_TIMEOUT: OnceLock<u64> = OnceLock::new();
        *OHKAMI_KEEPALIVE_TIMEOUT.get_or_init(|| {
            std::env::var("OHKAMI_KEEPALIVE_TIMEOUT").ok()
                .map(|v| v.parse().ok()).flatten()
                .unwrap_or(42)
        })
    }

    #[cfg(all(feature="ws", any(feature="rt_tokio",feature="rt_async-std")))]
    pub(crate) fn OHKAMI_WEBSOCKET_TIMEOUT() -> u64 {
        static OHKAMI_WEBSOCKET_TIMEOUT: OnceLock<u64> = OnceLock::new();
        *OHKAMI_WEBSOCKET_TIMEOUT.get_or_init(|| {
            std::env::var("OHKAMI_WEBSOCKET_TIMEOUT").ok()
                .map(|v| v.parse().ok()).flatten()
                .unwrap_or(1 * 60 * 60)
        })
    }
}

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

        match timeout_in(Duration::from_secs(env::OHKAMI_KEEPALIVE_TIMEOUT()), async {
            let mut req = Request::init();
            let mut req = unsafe {Pin::new_unchecked(&mut req)};
            loop {
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
                        let upgrade = res.send(&mut self.connection).await;

                        if !upgrade.is_none() {break upgrade}
                        if close {break Upgrade::None}

                        req.clear();
                    }
                    Ok(None) => break Upgrade::None,
                    Err(res) => {res.send(&mut self.connection).await;},
                };
            }
        }).await {
            Some(Upgrade::None) | None => {
                crate::DEBUG!("about to shutdown connection");
        
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

            #[cfg(all(feature="ws", any(feature="rt_tokio",feature="rt_async-std")))]
            Some(Upgrade::WebSocket((config, handler))) => {
                use crate::ws::{Connection, Message, CloseFrame, CloseCode};

                crate::DEBUG!("WebSocket session started");

                let mut conn = Connection::new(self.connection, config);

                let close = timeout_in(Duration::from_secs(env::OHKAMI_WEBSOCKET_TIMEOUT()),
                    handler(conn.clone())
                ).await;

                if !conn.is_closed() {
                    conn.send(Message::Close(Some(match close {
                        Some(_) => {
                            crate::DEBUG!("Closing WebSocket session...");
                            CloseFrame {
                                code:   CloseCode::Normal,
                                reason: None
                            }
                        }
                        None => {
                            crate::warning!("[WARNING] WebSocket session is aborted by `OHKAMI_WEBSOCKET_TIMEOUT` (default to 1 hour, and can be set via environment variable)");
                            CloseFrame {
                                code:   CloseCode::Library(4000),
                                reason: Some("OHKAMI_WEBSOCKET_TIMEOUT".into())
                            }
                        }
                    }))).await.expect("Failed to send close message");
                }

                crate::DEBUG!("WebSocket session finished");
            }
        }
    }
}
