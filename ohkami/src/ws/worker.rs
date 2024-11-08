use mews::{Message, CloseFrame, CloseCode};
use worker::{WebSocketPair, EventStream, wasm_bindgen_futures};

impl<'req> super::WebSocketContext<'req> {
    pub fn upgrade<H, F>(
        self,
        handler: H
    ) -> WebSocket
    where
        H: FnOnce(Connection) -> F,
        F: std::future::Future<Output = ()> + 'static
    {
        let WebSocketPair {
            client: session,
            server: ws
        } = WebSocketPair::new().expect("failed to create WebSocketPair");

        ws.accept().ok();
        wasm_bindgen_futures::spawn_local(handler(Connection::new(ws)));

        WebSocket { session }
    }
}

pub struct Connection {
    ws:     worker::WebSocket,
    events: Option<EventStream<'static>>
}
impl Connection {
    fn new(ws: worker::WebSocket) -> Self {
        Self { ws, events:None }
    }
}
impl Connection {
    pub async fn recv(&mut self) -> Result<Option<Message>, worker::Error> {
        use std::mem::transmute as unchecked_static;
        use ohkami_lib::StreamExt;
        use worker::{WebsocketEvent, worker_sys::web_sys::MessageEvent, js_sys::Uint8Array};

        if self.events.is_none() {
            crate::DEBUG!("[ws::Connection::recv] initial call: setting events");

            self.events = Some(match self.ws.events() {
                Ok(events) => unsafe {unchecked_static(events)},
                Err(error) => return Err(error)
            });
        }

        match (unsafe {self.events.as_mut().unwrap_unchecked()}).next().await {
            None            => Ok(None),
            Some(Err(err))  => Err(err),
            Some(Ok(event)) => Ok(Some(match event {
                WebsocketEvent::Close(event) => {
                    crate::DEBUG!("[ws::Connection::recv] close");
                    Message::Close(Some(CloseFrame {
                        code:   CloseCode::from_u16(event.code()),
                        reason: Some(event.reason().into())
                    }))
                }
                WebsocketEvent::Message(event) => {
                    let data = AsRef::<MessageEvent>::as_ref(&event).data();
                    if data.is_string() {
                        let data = data.as_string();
                        crate::DEBUG!("[ws::Connection::recv] data.is_string(): `{data:?}`");
                        Message::Text(data.ok_or(worker::Error::BadEncoding)?)
                    } else if data.is_object() {
                        let data = Uint8Array::new(&data).to_vec();
                        crate::DEBUG!("[ws::Connection::recv] not data.is_string() but data.is_object(): `{data:?}`");
                        Message::Binary(data)
                    } else {
                        crate::DEBUG!("[ws::Connection::recv] NOT data.is_object()");
                        return Err(worker::Error::Infallible)
                    }
                }
            }))
        }
    }

    #[inline]
    pub async fn send(&mut self, message: Message) -> Result<(), worker::Error> {
        match message {
            Message::Text(text)         => self.ws.send_with_str(text),
            Message::Binary(binary)     => self.ws.send_with_bytes(binary),
            Message::Close(None)        => self.ws.close::<&str>(None, None),
            Message::Close(Some(frame)) => self.ws.close(Some(frame.code.as_u16()), frame.reason),
            Message::Ping(_) | Message::Pong(_) => Err(worker::Error::RustError((|message| {
                format!("`Connection::send` got `{message:?}`, but sending ping/pong is not supported on `rt_worker`")
            })(message)))
        }
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
        // headers and something other
    }
}
impl From<worker::WebSocket> for WebSocket {
    fn from(session: worker::WebSocket) -> Self {
        Self { session }
    }
}
impl Into<worker::WebSocket> for WebSocket {
    fn into(self) -> worker::WebSocket {
        self.session
    }
}
impl Into<worker::Result<worker::Response>> for WebSocket {
    fn into(self) -> worker::Result<worker::Response> {
        worker::Response::from_websocket(self.session)
    }
}
