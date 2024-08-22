#![cfg(all(feature="ws", any(feature="rt_tokio",feature="rt_async-std",feature="rt_glommio")))]

mod connection;
mod message;
mod frame;

pub use message::{Message, CloseFrame};
pub use frame::{CloseCode};
pub use connection::Connection;
#[cfg(feature="rt_tokio")] pub use connection::split;

use std::{future::Future, pin::Pin};
use crate::{__rt__, FromRequest, IntoResponse, Request, Response};


/// # Context for WebSocket handshake
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::ws::{WebSocketContext, WebSocket, Message};
/// 
/// async fn ws(ctx: WebSocketContext<'_>) -> WebSocket {
///     ctx.connect(|mut ws| async move {
///         ws.send(Message::Text(
///             "Hello, WebSocket! and bye...".to_string()
///         )).await.expect("failed to send")
///     })
/// }
/// ```
pub struct WebSocketContext<'req> {
    sec_websocket_key: &'req str,
} const _: () = {
    impl<'req> FromRequest<'req> for WebSocketContext<'req> {
        type Error = Response;

        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            if !req.headers.Connection()?.contains("Upgrade") {
                return Some(Err((|| Response::BadRequest().with_text("upgrade request must have `Connection: Upgrade`"))()))
            }
            if req.headers.Upgrade()? != "websocket" {
                return Some(Err((|| Response::BadRequest().with_text("upgrade request must have `Upgrade: websocket`"))()))
            }
            if req.headers.SecWebSocketVersion()? != "13" {
                return Some(Err((|| Response::BadRequest().with_text("upgrade request must have `Sec-WebSocket-Version: 13`"))()))
            }

            req.headers.SecWebSocketKey().map(|sec_websocket_key|
                Ok(Self { sec_websocket_key })
            )
        }
    }

    impl<'ctx> WebSocketContext<'ctx> {
        pub fn connect<Fut: Future<Output = ()> + Send + 'static>(self,
            handler: impl FnOnce(Connection<__rt__::TcpStream>) -> Fut + Send + Sync + 'static
        ) -> WebSocket {
            self.connect_with(Config::default(), handler)
        }

        pub fn connect_with<Fut: Future<Output = ()> + Send + 'static>(self,
            config:  Config,
            handler: impl FnOnce(Connection<__rt__::TcpStream>) -> Fut + Send + Sync + 'static
        ) -> WebSocket {
            WebSocket {
                config,
                sec_websocket_key: sign(self.sec_websocket_key),
                handler: Box::new(move |ws| Box::pin({
                    let session = handler(ws);
                    async {session.await}
                }))
            }
        }
    }
};

pub(crate) type Handler = Box<dyn
    FnOnce(Connection<__rt__::TcpStream>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>
    + Send + Sync
>;

/// # Response for upgrading to WebSocket
/// 
/// Perform the handshake with a `WebSocketContext`,
/// establish a WebSocket connection,
/// and run the given handler.
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::ws::{WebSocketContext, WebSocket, Message};
/// 
/// async fn ws(ctx: WebSocketContext<'_>) -> WebSocket {
///     ctx.connect(|mut ws| async move {
///         ws.send(Message::Text(
///             "Hello, WebSocket! and bye...".to_string()
///         )).await.expect("failed to send")
///     })
/// }
/// ```
pub struct WebSocket {
    config:            Config,
    sec_websocket_key: String,
    handler:           Handler,
} impl IntoResponse for WebSocket {
    fn into_response(self) -> Response {
        Response::SwitchingProtocols().with_headers(|h|h
            .Connection("Upgrade")
            .Upgrade("websocket")
            .SecWebSocketAccept(self.sec_websocket_key)
        ).with_websocket(self.config, self.handler)
    }
}

/// ## Note
/// 
/// - Currently, subprotocols via `Sec-WebSocket-Protocol` is not supported
#[derive(Clone, Debug)]
pub struct Config {
    pub write_buffer_size:      usize,
    pub max_write_buffer_size:  usize,
    pub accept_unmasked_frames: bool,
    pub max_message_size:       Option<usize>,
    pub max_frame_size:         Option<usize>,
} const _: () = {
    impl Default for Config {
        fn default() -> Self {
            Self {
                write_buffer_size:      128 * 1024, // 128 KiB
                max_write_buffer_size:  usize::MAX,
                accept_unmasked_frames: false,
                max_message_size:       Some(64 << 20),
                max_frame_size:         Some(16 << 20),
            }
        }
    }
};

#[inline] fn sign(sec_websocket_key: &str) -> String {
    use ::sha1::{Sha1, Digest};
    use ohkami_lib::base64;

    let mut sha1 = <Sha1 as Digest>::new();
    sha1.update(sec_websocket_key.as_bytes());
    sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    base64::encode(sha1.finalize())
}

#[cfg(test)]
#[test] fn test_sign() {
    /* example in https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers#server_handshake_response */
    assert_eq!(sign("dGhlIHNhbXBsZSBub25jZQ=="), "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
}
