pub use ohkami_lib::stream::{Stream, StreamExt};

use worker::{WebSocketPair, wasm_bindgen_futures};

impl<'req> super::WebSocketContext<'req> {
    pub fn upgrade<H, F>(
        self,
        handler: H
    ) -> WebSocket
    where
        H: FnOnce(worker::WebSocket) -> F,
        F: std::future::Future<Output = ()> + 'static
    {
        let WebSocketPair {
            client: session,
            server: ws
        } = WebSocketPair::new().expect("failed to create WebSocketPair");

        ws.accept().ok();
        wasm_bindgen_futures::spawn_local(handler(ws));

        WebSocket { session }
    }
}

pub type Session = ::worker::WebSocket;

pub struct WebSocket {
    session: Session
}
impl crate::IntoResponse for WebSocket {
    fn into_response(self) -> crate::Response {
        crate::Response::SwitchingProtocols().with_websocket(self.session)
        // let `worker` crate and Cloudflare Workers to do around
        // `Sec-WebSocket-Accept` and other headers
    }
}
