use std::{future::Future, borrow::Cow};
use super::websocket::Config;
use super::{WebSocket, sign::Sha1};
use crate::{Response, Request, utils};
use crate::__rt__::{task};
use crate::http::{Method};
use crate::layer0_lib::{base64};
use super::{assume_upgradable, UpgradeID};


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
            return Err((|| utils::Text::BadRequest("Method is not `GET`").into())())
        }
        if req.headers.Connection() != Some("upgrade") {
            return Err((|| utils::Text::BadRequest("Connection header is not `upgrade`").into())())
        }
        if req.headers.Upgrade() != Some("websocket") {
            return Err((|| utils::Text::BadRequest("Upgrade header is not `websocket`").into())())
        }
        if req.headers.SecWebSocketVersion() != Some("13") {
            return Err((|| utils::Text::BadRequest("Sec-WebSocket-Version header is not `13`").into())())
        }

        let sec_websocket_key = Cow::Owned(req.headers.SecWebSocketKey()
            .ok_or_else(|| utils::Text::BadRequest("Sec-WebSocket-Key header is missing").into())?
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
        fn sign(sec_websocket_key: &str) -> String {
            let mut sha1 = Sha1::new();
            sha1.write(sec_websocket_key.as_bytes());
            sha1.write(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
            base64::encode(sha1.sum())
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
