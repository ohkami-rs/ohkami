pub(crate) struct Config {
    #[cfg(feature="__rt_native__")]
    keepalive_timeout: std::sync::LazyLock<u64>,

    #[cfg(feature="__rt_native__")]
    #[cfg(feature="ws")]
    websocket_timeout: std::sync::LazyLock<u64>,

    #[cfg(feature="openapi")]
    openapi_metadata: std::sync::OnceLock<OpenAPIMetadata>,
}
impl Config {
    pub(crate) fn keepalive_timeout(&self) -> u64 {
        *self.keepalive_timeout
    }
    pub(crate) fn websocket_timeout(&self) -> u64 {
        *self.websocket_timeout
    }
    pub(crate) fn openapi_metadata(&self) -> &std::sync::OnceLock<OpenAPIMetadata> {
        &self.openapi_metadata
    }
}
impl Config {
    pub(super) const fn new() -> Self {
        Self {
            #[cfg(feature="__rt_native__")]
            keepalive_timeout: std::sync::LazyLock::new(|| std::env::var("OHKAMI_KEEPALIVE_TIMEOUT")
                .ok().map(|v| v.parse().ok()).flatten()
                .unwrap_or(42)
            ),
            #[cfg(feature="__rt_native__")]
            #[cfg(feature="ws")]
            websocket_timeout: std::sync::LazyLock::new(|| std::env::var("OHKAMI_WEBSOCKET_TIMEOUT")
                .ok().map(|v| v.parse().ok()).flatten()
                .unwrap_or(42)
            ),
            #[cfg(feature="openapi")]
            openapi_metadata: std::sync::OnceLock::new()
        }
    }
}

#[cfg(feature="openapi")]
#[derive(Clone)]
pub(crate) struct OpenAPIMetadata {
    pub(crate) file_path: std::path::PathBuf,
    pub(crate) title:     &'static str,
    pub(crate) version:   &'static str,
    pub(crate) servers:   Vec<crate::openapi::document::Server>,
}
