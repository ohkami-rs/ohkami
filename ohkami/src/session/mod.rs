#![cfg(feature="__rt_native__")]

use std::{any::Any, pin::Pin, sync::Arc, time::Duration};
use std::panic::{AssertUnwindSafe, catch_unwind};
use crate::__rt__::{AsyncRead, AsyncWrite};
use crate::response::Upgrade;
use crate::util::timeout_in;
use crate::router::r#final::Router;
use crate::{Request, Response};

#[cfg(feature="ws")]
use crate::__rt__::TcpStream;

pub(crate) struct Session<S> {
    router: Arc<Router>,
    connection: S,
    ip: std::net::IpAddr,
}

#[cfg(feature="ws")]
pub(crate) trait WebSocketUpgradeable {
    fn into_websocket_stream(self) -> Result<TcpStream, &'static str>;
}

#[cfg(feature="ws")]
impl WebSocketUpgradeable for TcpStream {
    fn into_websocket_stream(self) -> Result<TcpStream, &'static str> {
        Ok(self)
    }
}

impl<S> Session<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static + WebSocketUpgradeable,
{
    pub(crate) fn new(
        router: Arc<Router>,
        connection: S,
        ip: std::net::IpAddr
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
                crate::WARNING!("[Panicked]: {msg}");
            } else if let Some(msg) = panic.downcast_ref::<&str>() {
                crate::WARNING!("[Panicked]: {msg}");
            } else {
                crate::WARNING!("[Panicked]");
            }
            crate::Response::InternalServerError()
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
                        let upgrade = res.send(&mut self.connection).await;

                        if !upgrade.is_none() {break upgrade}
                        if close {break Upgrade::None}
                    }
                    Ok(None) => break Upgrade::None,
                    Err(res) => {res.send(&mut self.connection).await;},
                }
            }
        }).await {
            None => crate::WARNING!("[WARNING] \
                Session timeouted. In Ohkami, Keep-Alive timeout \
                is set to 42 seconds by default and is configurable \
                by `OHKAMI_KEEPALIVE_TIMEOUT` environment variable.\
            "),

            Some(Upgrade::None) => crate::DEBUG!("about to shutdown connection"),

            #[cfg(feature="ws")]
            Some(Upgrade::WebSocket(ws)) => {
                match self.connection.into_websocket_stream() {
                    Ok(tcp_stream) => {
                        crate::DEBUG!("WebSocket session started");

                        let aborted = ws.manage_with_timeout(
                            Duration::from_secs(crate::CONFIG.websocket_timeout()),
                            tcp_stream
                        ).await;
                        if aborted {
                            crate::WARNING!("[WARNING] \
                                WebSocket session aborted by timeout. In Ohkami, \
                                WebSocket timeout is set to 3600 seconds (1 hour) \
                                by default and is configurable by `OHKAMI_WEBSOCKET_TIMEOUT` \
                                environment variable.\
                            ");
                        }

                        crate::DEBUG!("WebSocket session finished");
                    }
                    Err(msg) => {
                        crate::WARNING!("[WARNING] {}", msg);
                    }
                }
            }
        }
    }
}