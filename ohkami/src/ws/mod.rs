#![cfg(all(feature="ws", feature="__rt_native__"))]

pub use mews::{
    Message,
    CloseCode, CloseFrame,
    Config,
    Connection,
    ReadHalf, WriteHalf,
    connection,
    split,
};

use crate::{__rt__, FromRequest, IntoResponse, Request, Response};


/// # Context for WebSocket handshake
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::ws::{WebSocketContext, WebSocket, Connection};
/// 
/// async fn ws(ctx: WebSocketContext<'_>) -> WebSocket {
///     ctx.upgrade(|mut conn: Connection| async move {
///         conn.send("Hello, WebSocket! and bye...").await
///     })
/// }
/// ```
pub struct WebSocketContext<'req> {
    sec_websocket_key: &'req str,
}
const _: () = {
    impl<'req> FromRequest<'req> for WebSocketContext<'req> {
        type Error = Response;

        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            if {
                let connection = req.headers.Connection()?;
                !(connection.contains("Upgrade") || connection.contains("upgrade"))
            } {
                return Some(Err((|| Response::BadRequest().with_text("upgrade request must have `Connection: Upgrade`"))()))
            }
            if req.headers.Upgrade()?.eq_ignore_ascii_case("websocket") {
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
        /// create a `WebSocket` with the handler and default `Config`.
        /// use [`upgrade_with`](WebSocketContext::upgrade_with) to provide a custom config.
        /// 
        /// ## handler
        /// 
        /// any `FnOnce + Send + Sync` returning `Future + Send` with following signature:
        /// 
        /// * `(Connection) -> () | std::io::Result<()>`
        /// * `(ReadHalf, WriteHalf) -> () | std::io::Result<()>`
        pub fn upgrade<T, C: mews::connection::UnderlyingConnection>(self,
            handler: impl mews::handler::IntoHandler<C, T>
        ) -> WebSocket<C> {
            self.upgrade_with(Config::default(), handler)
        }

        /// create a `WebSocket` with the config and handler.
        /// 
        /// ## handler
        /// 
        /// any `FnOnce + Send + Sync` returning `Future + Send` with following signature:
        /// 
        /// * `(Connection) -> () | std::io::Result<()>`
        /// * `(ReadHalf, WriteHalf) -> () | std::io::Result<()>`
        pub fn upgrade_with<T, C: mews::connection::UnderlyingConnection>(self,
            config:  Config,
            handler: impl mews::handler::IntoHandler<C, T>
        ) -> WebSocket<C> {
            let (sign, session) = mews::WebSocketContext::new(self.sec_websocket_key)
                .with(config)
                .on_upgrade(handler);
            WebSocket { sign, session }
        }
    }
};

/// # Response for upgrading to WebSocket
/// 
/// Perform handshake with a `WebSocketContext`,
/// establish a WebSocket connection,
/// and run the given handler.
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// use ohkami::ws::{WebSocketContext, WebSocket, Connection};
/// 
/// async fn ws(ctx: WebSocketContext<'_>) -> WebSocket {
///     ctx.upgrade(|mut conn: Connection| async move {
///         conn.send("Hello, WebSocket! and bye...").await
///     })
/// }
/// ```
/// 
/// <br>
/// 
/// *split_example.rs*
/// ```
/// # use tokio::{join, spawn};
/// # use tokio::time::{Duration, sleep};
/// # 
/// use ohkami::ws::{WebSocketContext, WebSocket, Message, ReadHalf, WriteHalf};
/// 
/// async fn ws(ctx: WebSocketContext<'_>) -> WebSocket {
///     ctx.upgrade(|mut r: ReadHalf, mut w: WriteHalf| async {
///         tokio::join!( /* joining is necessary to prevent resource leak or unsafe situations */
///             tokio::spawn(async move {
///                 while let Some(Message::Text(
///                     text
///                 )) = r.recv().await.expect("failed to recieve") {
///                     println!("[->] {text}");
///                 }
///             }),
///             tokio::spawn(async move {
///                 for text in ["abc", "def", "ghi", "jk", "lmno", "pqr", "stuvw", "xyz"] {
///                     w.send(text).await.expect("failed to send text");
///                     sleep(Duration::from_secs(1)).await;
///                 }
///             })
///         );
///     })
/// }
/// ```
pub struct WebSocket<C: mews::connection::UnderlyingConnection = __rt__::TcpStream> {
    sign:    String,
    session: mews::WebSocket<C>,
}
impl IntoResponse for WebSocket {
    fn into_response(self) -> Response {
        Response::SwitchingProtocols().with_headers(|h|h
            .Connection("Upgrade")
            .Upgrade("websocket")
            .SecWebSocketAccept(self.sign)
        ).with_websocket(self.session)
    }
}
