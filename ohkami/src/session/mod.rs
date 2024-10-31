#![cfg(feature="__rt_native__")]

use std::{any::Any, pin::Pin, sync::Arc, time::Duration};
use std::panic::{AssertUnwindSafe, catch_unwind};
use crate::__rt__::TcpStream;
use crate::response::Upgrade;
use crate::util::timeout_in;
use crate::ohkami::router::RadixRouter;
use crate::{Request, Response};


mod env {
    #![allow(unused, non_snake_case)]

    use std::sync::OnceLock;

    pub(crate) fn OHKAMI_KEEPALIVE_TIMEOUT() -> u64 {
        static OHKAMI_KEEPALIVE_TIMEOUT: OnceLock<u64> = OnceLock::new();
        *OHKAMI_KEEPALIVE_TIMEOUT.get_or_init(|| {
            std::env::var("OHKAMI_KEEPALIVE_TIMEOUT").ok()
                .map(|v| v.parse().ok()).flatten()
                .unwrap_or(42)
        })
    }

    #[cfg(feature="ws")]
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
    ip:         std::net::IpAddr,
}
impl Session {
    pub(crate) fn new(
        router:     Arc<RadixRouter>,
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
                        let upgrade = res.send(&mut self.connection).await;

                        if !upgrade.is_none() {break upgrade}
                        if close {break Upgrade::None}
                    }
                    Ok(None) => break Upgrade::None,
                    Err(res) => {res.send(&mut self.connection).await;},
                }
            }
        }).await {
            None => crate::warning!("[WARNING] \
                Session timeouted. In Ohkami, Keep-Alive timeout \
                is set to 42 seconds by default and is configurable \
                by `OHKAMI_KEEPALIVE_TIMEOUT` environment variable.\
            "),

            Some(Upgrade::None) => crate::DEBUG!("about to shutdown connection"),

            #[cfg(feature="ws")]
            Some(Upgrade::WebSocket(ws)) => {
                crate::DEBUG!("WebSocket session started");

                let aborted = ws.manage_with_timeout(
                    Duration::from_secs(env::OHKAMI_WEBSOCKET_TIMEOUT()),
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
