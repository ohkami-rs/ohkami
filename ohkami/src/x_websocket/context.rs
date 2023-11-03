use std::{future::Future, borrow::Cow};
use super::{WebSocket, sign};
use crate::{Response, Context, __rt__, Request};
use crate::http::{Method};


pub struct WebSocketContext<FU: OnFailedUpgrade = DefaultOnFailedUpgrade> {
    c:                      Context,
    config:                 Config,

    on_failed_upgrade:      FU,

    selected_protocol:      Option<Cow<'static, str>>,
    sec_websocket_key:      Cow<'static, str>,
    sec_websocket_protocol: Option<Cow<'static, str>>,
}

pub struct Config {
    write_buffer_size:      usize,
    max_write_buffer_size:  usize,
    max_message_size:       Option<usize>,
    max_frame_size:         Option<usize>,
    accept_unmasked_frames: bool,
} const _: () = {
    impl Default for Config {
        fn default() -> Self {
            Self {
                write_buffer_size:      128 * 1024, // 128 KiB
                max_write_buffer_size:  usize::MAX,
                max_message_size:       Some(64 << 20),
                max_frame_size:         Some(16 << 20),
                accept_unmasked_frames: false,
            }
        }
    }
};

pub enum UpgradeError { /* TODO */ }
pub trait OnFailedUpgrade: Send + 'static {
    fn handle(self, error: UpgradeError);
}
pub struct DefaultOnFailedUpgrade; const _: () = {
    impl OnFailedUpgrade for DefaultOnFailedUpgrade {
        fn handle(self, _: UpgradeError) { /* DO NOTHING (discard error) */ }
    }
};


impl WebSocketContext {
    pub(crate) fn new(c: Context, req: &mut Request) -> Result<Self, Cow<'static, str>> {
        if req.method() != Method::GET {
            return Err(Cow::Borrowed("Method is not `GET`"))
        }
        if req.header("Connection") != Some("upgrade") {
            return Err(Cow::Borrowed("Connection header is not `upgrade`"))
        }
        if req.header("Upgrade") != Some("websocket") {
            return Err(Cow::Borrowed("Upgrade header is not `websocket`"))
        }
        if req.header("Sec-WebSocket-Version") != Some("13") {
            return Err(Cow::Borrowed("Sec-WebSocket-Version header is not `13`"))
        }

        let sec_websocket_key = Cow::Owned(req.header("Sec-WebSocket-Key")
            .ok_or(Cow::Borrowed("Sec-WebSocket-Key header is missing"))?
            .to_string());

        let sec_websocket_protocol = req.header("Sec-WebSocket-Protocol")
            .map(|swp| Cow::Owned(swp.to_string()));

        Ok(Self {c,
            config:            Config::default(),
            on_failed_upgrade: DefaultOnFailedUpgrade,
            selected_protocol: None,
            sec_websocket_key,
            sec_websocket_protocol,
        })
    }
}

impl<FU: OnFailedUpgrade> WebSocketContext<FU> {
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
}

impl<FU: OnFailedUpgrade> WebSocketContext<FU> {
    pub fn protocols<S: Into<Cow<'static, str>>>(mut self, protocols: impl Iterator<Item = S>) -> Self {
        if let Some(req_protocols) = &self.sec_websocket_protocol {
            self.selected_protocol = protocols.map(Into::into)
                .find(|p| req_protocols.split(',').any(|req_p| req_p.trim() == p))
        }
        self
    }
}

impl<FU: OnFailedUpgrade> WebSocketContext<FU> {
    pub fn on_upgrade<
        Fut: Future<Output = ()> + Send + 'static,
    >(
        self,
        callback: impl Fn(WebSocket) -> Fut + Send + 'static
    ) -> Response {
        let Self {
            mut c,
            config,
            on_failed_upgrade,
            selected_protocol,
            sec_websocket_key,
            sec_websocket_protocol,
        } = self;

        __rt__::task::spawn(async move {
            todo!()
        });

        c.headers
            .custom("Connection", "Upgrade")
            .custom("Upgrade", "websocket")
            .custom("Sec-WebSocket-Accept", sign(&sec_websocket_key));
        if let Some(protocol) = selected_protocol {
            c.headers
                .custom("Sec-WebSocket-Protocol", protocol);
        }
        c.SwitchingProtocols()
    }
}

fn sign(sec_websocket_key: &str) -> String {
    let mut sha1 = sign::Sha1::new();
    sha1.write(sec_websocket_key.as_bytes());
    sha1.write(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");

    let sec_websocket_accept_bytes = sign::encode_sha1_to_base64(sha1.sum());
    unsafe {String::from_utf8_unchecked(sec_websocket_accept_bytes.to_vec())}
}
