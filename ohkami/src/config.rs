use std::sync::{OnceLock, LazyLock};
use std::path::PathBuf;
use std::env;

pub(crate) struct Config {
    #[cfg(feature="__rt_native__")]
    keepalive_timeout: LazyLock<u64>,

    #[cfg(feature="__rt_native__")]
    #[cfg(feature="ws")]
    websocket_timeout: LazyLock<u64>,

    #[cfg(feature="openapi")]
    openapi_filepath: OnceLock<PathBuf>,
}

impl Config {
    pub(crate) fn keepalive_timeout(&self) -> u64 {
        *self.keepalive_timeout
    }
    pub(crate) fn websocket_timeout(&self) -> u64 {
        *self.websocket_timeout
    }
    pub(crate) fn openapi_filepath(&self) -> &OnceLock<PathBuf> {
        &self.openapi_filepath
    }
}

impl Config {
    pub(super) const fn new() -> Self {
        Self {
            #[cfg(feature="__rt_native__")]
            keepalive_timeout: LazyLock::new(|| env::var("OHKAMI_KEEPALIVE_TIMEOUT")
                .ok().map(|v| v.parse().ok()).flatten()
                .unwrap_or(42)
            ),
            #[cfg(feature="__rt_native__")]
            #[cfg(feature="ws")]
            websocket_timeout: LazyLock::new(|| env::var("OHKAMI_WEBSOCKET_TIMEOUT")
                .ok().map(|v| v.parse().ok()).flatten()
                .unwrap_or(42)
            ),
            #[cfg(feature="openapi")]
            openapi_filepath: OnceLock::new()
        }
    }
}
