pub(crate) struct Config {
    #[cfg(feature="__rt_native__")]
    request_bufsize: std::sync::LazyLock<usize>,

    #[cfg(feature="__rt_native__")]
    keepalive_timeout: std::sync::LazyLock<u64>,

    #[cfg(feature="__rt_native__")]
    #[cfg(feature="ws")]
    websocket_timeout: std::sync::LazyLock<u64>,
}

impl Config {
    #[cfg(feature="__rt_native__")]
    #[inline]
    pub(crate) fn request_bufsize(&self) -> usize {
        *(&*self.request_bufsize)
    }

    #[cfg(feature="__rt_native__")]
    #[inline]
    pub(crate) fn keepalive_timeout(&self) -> u64 {
        *(&*self.keepalive_timeout)
    }

    #[cfg(feature="__rt_native__")]
    #[cfg(feature="ws")]
    pub(crate) fn websocket_timeout(&self) -> u64 {
        *(&*self.websocket_timeout)
    }
}

impl Config {
    pub(super) const fn new() -> Self {
        Self {
            #[cfg(feature="__rt_native__")]
            request_bufsize: std::sync::LazyLock::new(|| std::env::var("OHKAMI_REQUEST_BUFSIZE")
                .ok()
                .map(|v| v.parse().ok())
                .flatten()
                .unwrap_or(1 << 11)
            ),

            #[cfg(feature="__rt_native__")]
            keepalive_timeout: std::sync::LazyLock::new(|| std::env::var("OHKAMI_KEEPALIVE_TIMEOUT")
                .ok()
                .map(|v| v.parse().ok())
                .flatten()
                .unwrap_or(30) // 30 seconds
            ),

            #[cfg(feature="__rt_native__")]
            #[cfg(feature="ws")]
            websocket_timeout: std::sync::LazyLock::new(|| std::env::var("OHKAMI_WEBSOCKET_TIMEOUT")
                .ok()
                .map(|v| v.parse().ok())
                .flatten()
                .unwrap_or(1 * 60 * 60) // 1 hour
            ),
        }
    }
}
