use std::{future::Future, borrow::Cow};
use super::{WebSocket};
use crate::{Response};


pub struct WebSocketContext<FU: OnFailedUpgrade = DefaultOnFailedUpgrade> {
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
pub trait OnFailedUpgrade: Send + 'static {
    fn handle(self, error: UpgradeError);
}
pub struct UpgradeError { /* TODO */ }
pub struct DefaultOnFailedUpgrade; const _: () = {
    impl OnFailedUpgrade for DefaultOnFailedUpgrade {
        fn handle(self, _: UpgradeError) { /* DO NOTHING (discard error) */ }
    }
};

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
        todo!()
    }
}
