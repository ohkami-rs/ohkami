use std::{future::Future, borrow::Cow};
use super::{assume_upgradable, UpgradeID};
use super::websocket::Config;
use super::{WebSocket};
use crate::{Response, Request};
use crate::__rt__::{task};
use crate::http::{Method};
use crate::layer0_lib::{base64};


pub struct WebSocketContext<UFH: UpgradeFailureHandler = DefaultUpgradeFailureHandler> {
    id:                     Option<UpgradeID>,
    config:                 Config,

    on_failed_upgrade:      UFH,

    sec_websocket_key:      Cow<'static, str>,
    selected_protocol:      Option<Cow<'static, str>>,
    sec_websocket_protocol: Option<Cow<'static, str>>,
}

pub trait UpgradeFailureHandler {
    fn handle(self, error: UpgradeError);
}
pub enum UpgradeError {
    NotRequestedUpgrade,
}
pub struct DefaultUpgradeFailureHandler;
impl UpgradeFailureHandler for DefaultUpgradeFailureHandler {
    fn handle(self, _: UpgradeError) {/* discard error */}
}

impl WebSocketContext {
    pub(crate) fn new(req: &mut Request) -> Result<Self, Response> {
        if req.method != Method::GET {
            return Err((|| Response::BadRequest().text("Method is not `GET`"))())
        }
        if req.headers.Connection() != Some("upgrade") {
            return Err((|| Response::BadRequest().text("Connection header is not `upgrade`"))())
        }
        if req.headers.Upgrade() != Some("websocket") {
            return Err((|| Response::BadRequest().text("Upgrade header is not `websocket`"))())
        }
        if req.headers.SecWebSocketVersion() != Some("13") {
            return Err((|| Response::BadRequest().text("Sec-WebSocket-Version header is not `13`"))())
        }

        let sec_websocket_key = Cow::Owned(req.headers.SecWebSocketKey()
            .ok_or_else(|| Response::BadRequest().text("Sec-WebSocket-Key header is missing"))?
            .to_string());

        let sec_websocket_protocol = req.headers.SecWebSocketProtocol()
            .map(|swp| Cow::Owned(swp.to_string()));

        Ok(Self {
            id:                req.upgrade_id,
            config:            Config::default(),
            on_failed_upgrade: DefaultUpgradeFailureHandler,
            selected_protocol: None,
            sec_websocket_key,
            sec_websocket_protocol,
        })
    }
}

impl WebSocketContext {
    pub fn write_buffer_size(mut self, size: usize) -> Self {
        self.config.write_buffer_size = size;
        self
    }
    pub fn max_write_buffer_size(mut self, size: usize) -> Self {
        self.config.max_write_buffer_size = size;
        self
    }
    pub fn max_message_size(mut self, size: usize) -> Self {
        self.config.max_message_size = Some(size);
        self
    }
    pub fn max_frame_size(mut self, size: usize) -> Self {
        self.config.max_frame_size = Some(size);
        self
    }
    pub fn accept_unmasked_frames(mut self) -> Self {
        self.config.accept_unmasked_frames = true;
        self
    }

    pub fn protocols<S: Into<Cow<'static, str>>>(mut self, protocols: impl Iterator<Item = S>) -> Self {
        if let Some(req_protocols) = &self.sec_websocket_protocol {
            self.selected_protocol = protocols.map(Into::into)
                .find(|p| req_protocols.split(',').any(|req_p| req_p.trim() == p))
        }
        self
    }
}

impl WebSocketContext {
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
            on_failed_upgrade,
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
