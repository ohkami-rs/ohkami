#![cfg(feature="ws")]

#[cfg(feature="__rt_native__")]
mod native;
#[cfg(feature="__rt_native__")]
pub use self::native::*;

#[cfg(feature="rt_worker")]
mod worker;
#[cfg(feature="rt_worker")]
pub use self::worker::*;

/// # Context for WebSocket handshake
/// 
/// `.upgrade` performs handshake and creates a WebSocket session.
/// 
/// ### note
/// 
/// On native runtimes, the session is timeout in 3600 seconds ( = 1 hour )
/// by default. This is configurable by `OHKAMI_WEBSOCKET_TIMEOUT`
/// environment variable.
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::ws::{WebSocketContext, WebSocket};
/// 
/// async fn ws(ctx: WebSocketContext<'_>) -> WebSocket {
///     ctx.upgrade(|mut conn| async move {
///         conn.send("Hello, WebSocket! and bye...").await
///             .expect("failed to send")
///     })
/// }
/// ```
pub struct WebSocketContext<'req> {
    #[allow(unused/* on rt_worker */)]
    sec_websocket_key: &'req str,
}

impl<'req> crate::FromRequest<'req> for WebSocketContext<'req> {
    type Error = crate::Response;

    #[inline]
    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        if !matches!(req.headers.Connection()?, "Upgrade" | "upgrade") {
            return Some(Err((|| crate::Response::BadRequest().with_text("upgrade request must have `Connection: Upgrade`"))()))
        }
        if !(req.headers.Upgrade()?.eq_ignore_ascii_case("websocket")) {
            return Some(Err((|| crate::Response::BadRequest().with_text("upgrade request must have `Upgrade: websocket`"))()))
        }
        if !(req.headers.SecWebSocketVersion()? == "13") {
            return Some(Err((|| crate::Response::BadRequest().with_text("upgrade request must have `Sec-WebSocket-Version: 13`"))()))
        }

        req.headers.SecWebSocketKey().map(|sec_websocket_key|
            Ok(Self { sec_websocket_key })
        )
    }
}

impl<'req> WebSocketContext<'req> {
    pub fn new(sec_websocket_key: &'req str) -> Self {
        Self { sec_websocket_key }
    }
}
