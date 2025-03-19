#[cfg(feature = "__rt_native__")]
use crate::router::r#final::Router;
use crate::response::Upgrade;
use crate::util::timeout_in;
use crate::{Request, Response};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::{any::Any, pin::Pin, sync::Arc, time::Duration};

#[cfg(feature = "rt_tokio")]
use tokio::io::{AsyncRead, AsyncWrite};
#[cfg(any(feature = "rt_async-std", feature = "rt_smol"))]
use futures_util::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "rt_glommio")]
use glommio::io::{AsyncRead, AsyncWrite};

pub(crate) struct Session<S> {
    router: Arc<Router>,
    connection: S, // Changed connection to generic type for TcpStream and TlsStream
    ip: std::net::IpAddr,
}

impl<S> Session<S> 
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub(crate) fn new(router: Arc<Router>, connection: S, ip: std::net::IpAddr) -> Self {
        Self {
            router,
            connection,
            ip,
        }
    }

    pub(crate) async fn manage(mut self) {
        #[cold] #[inline(never)]
        fn panicking(panic: Box<dyn Any + Send>) -> Response {
            if let Some(msg) = panic.downcast_ref::<String>() {
                crate::WARNING!("panic: {msg}");
            } else if let Some(msg) = panic.downcast_ref::<&str>() {
                crate::WARNING!("panic: {msg}");
            } else {
                crate::WARNING!("panic");
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
            None => crate::WARNING!("\
                Session timeouted. In Ohkami, Keep-Alive timeout \
                is set to 42 seconds by default and is configurable \
                by `OHKAMI_KEEPALIVE_TIMEOUT` environment variable.\
            "),

            Some(Upgrade::None) => crate::DEBUG!("about to shutdown connection"),

            #[cfg(feature="ws")]
            Some(Upgrade::WebSocket(ws)) => {
                crate::DEBUG!("WebSocket session started");

                let aborted = ws.manage_with_timeout(
                    Duration::from_secs(crate::CONFIG.websocket_timeout()),
                    self.connection
                ).await;
                if aborted {
                    crate::WARNING!("\
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