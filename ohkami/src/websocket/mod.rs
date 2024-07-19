#![cfg(all(
    feature="ws",
))]

mod session;
mod message;
mod frame;

pub use message::Message;
pub(crate) use session::WebSocket as Session;

use ohkami_lib::base64;
use std::{future::Future, borrow::Cow, pin::Pin};
use crate::{FromRequest, IntoResponse, Method, Request, Response};
use crate::__rt__::{task, AsyncReader, AsyncWriter, TcpStream};


// #[derive(Clone)]
pub struct WebSocketContext<'req> {
    sec_websocket_key: &'req str,
} const _: () = {
    impl<'req> FromRequest<'req> for WebSocketContext<'req> {
        type Error = std::convert::Infallible;

        fn from_request(req: &'req Request) -> Option<Result<Self, Self::Error>> {
            req.headers.SecWebSocketKey().map(|swk| Ok(Self {
                sec_websocket_key: swk,
            }))
        }
    }

    impl<'ws> WebSocketContext<'ws> {
        pub fn connect<Fut: Future<Output = ()> + Send + 'static>(self,
            handler: impl Fn(Session<'ws, TcpStream>) -> Fut + Send + Sync + 'static
        ) -> WebSocket {
            #[inline] fn signed(sec_websocket_key: &str) -> String {
                use ::sha1::{Sha1, Digest};
                let mut sha1 = <Sha1 as Digest>::new();
                sha1.update(sec_websocket_key.as_bytes());
                sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
                base64::encode(sha1.finalize())
            }

            WebSocket {
                sec_websocket_key: signed(self.sec_websocket_key),
                handler: Box::new(move |ws| Box::pin({
                    let h = handler(unsafe {std::mem::transmute::<_, Session<'ws, _>>(ws)});
                    async {h.await}
                }))
            }
        }
    }
};

pub(crate) type Handler = Box<dyn
    Fn(Session<'_, TcpStream>) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>
    + Send + Sync + 'static
>;

// #[derive(Clone)]
pub struct WebSocket {
    sec_websocket_key: String,
    handler:           Handler,
} impl IntoResponse for WebSocket {
    fn into_response(self) -> Response {
        Response::SwitchingProtocols().with_headers(|h|h
            .Connection("Update")
            .Upgrade("websocket")
            .SecWebSocketAccept(self.sec_websocket_key)
        ).with_websocket(self.handler)
    }
}

/// ## Note
/// 
/// - Currently, subprotocols with `Sec-WebSocket-Protocol` is not supported
//#[derive(Clone)]
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

// impl WebSocket {
//     /// shortcut for `WebSocket::with(Config::default())`
//     pub fn new<Fut: Future<Output = ()> + Send>(
//         handler: impl Fn(Session<'_, TcpStream>) -> Fut + 'static
//     ) -> Self {
//         Self::with(Config::default(), handler)
//     }
// 
//     pub fn with<Fut: Future<Output = ()> + Send>(
//         config:  Config,
//         handler: impl Fn(Session<'_, TcpStream>) -> Fut + 'static
//     ) -> Self {
//         task::spawn(async move {
//             todo!()
//         });
// 
//         Self { config, handler:  }
//     }
// }
// 
// 

/*
impl WebSocket {
    pub fn on_upgrade<Fut: Future<Output = ()> + Send + 'static>(
        self,
        handler: impl Fn(WebSocket) -> Fut + Send + Sync + 'static
    ) -> Response {
        #[inline] fn sign(sec_websocket_key: &str) -> String {
            use ::sha1::{Sha1, Digest};

            let mut sha1 = <Sha1 as Digest>::new();
            sha1.update(sec_websocket_key.as_bytes());
            sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
            base64::encode(sha1.finalize())
        }

        let Self {
            config,
            selected_protocol,
            sec_websocket_key,
            ..
        } = self;

        task::spawn({
            async move {
                let stream = match self.id {
                    None     => return on_failed_upgrade.handle(UpgradeError::NotRequestedUpgrade),
                    Some(id) => assume_upgradable(id).await,
                };

                let ws = WebSocket::new(stream, config);
                handler(ws).await
            }
        });

        let mut handshake_res = Response::SwitchingProtocols();
        handshake_res.headers.set()
            .Connection("Update")
            .Upgrade("websocket")
            .SecWebSocketAccept(sign(&sec_websocket_key));
        if let Some(protocol) = selected_protocol {
            handshake_res.headers.set()
                .SecWebSocketProtocol(protocol.to_string());
        }
        handshake_res
    }
}
*/
