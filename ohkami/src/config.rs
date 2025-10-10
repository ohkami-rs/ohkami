/// Configuration for Ohkami server.
///
/// 1. By default, the default values will be used.
/// 2. Each field can be overridden by the corresponding environment variable.
///
/// ```no_run
/// # fn my_ohkami() -> ohkami::Ohkami { todo!() }
/// #
/// #[tokio::main]
/// async fn main() {
///     my_ohkami().howl_with(ohkami::Config {
///         request_bufsize: 4096, // 4 KiB
///         ..ohkami::Config::new()
///     }, "0.0.0.0:5000").await;
/// }
/// ```
#[derive(Clone, Copy)]
pub struct Config {
    /// [bytes] size of the internal buffer used to read requests.
    ///
    /// - default: 2048 (2 KiB)
    /// - env: `OHKAMI_REQUEST_BUFSIZE`
    #[cfg(feature = "__rt_native__")]
    pub request_bufsize: usize,

    /// [bytes] maximum size of the request payload.
    ///
    /// - default: 4294967296 (4 GiB)
    /// - env: `OHKAMI_REQUEST_PAYLOAD_LIMIT`
    #[cfg(feature = "__rt_native__")]
    pub request_payload_limit: usize,

    /// [secs] duration of the keep-alive timeout.
    ///
    /// - default: 30 (30 seconds)
    /// - env: `OHKAMI_KEEPALIVE_TIMEOUT`
    #[cfg(feature = "__rt_native__")]
    pub keepalive_timeout: u64,

    /// [secs] duration of the WebSocket session timeout.
    ///
    /// - default: 3600 (1 hour)
    /// - env: `OHKAMI_WEBSOCKET_TIMEOUT`
    #[cfg(feature = "__rt_native__")]
    #[cfg(feature = "ws")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ws")))]
    pub websocket_timeout: u64,

    #[doc(hidden)]
    pub __private__: (),
}

impl Config {
    /// Default configuration values.
    ///
    /// Intentionally NOT implementing `Default` trait
    /// in order to prevent misuses like:
    ///
    /// ```ignore
    /// Ohkami::new(...).howl_with(ohkami::Config {
    ///     request_bufsize: 4096,
    ///     ..Default::default() // <--- This does NOT read from env vars!
    ///     // Here `..Config::new()` should be used
    ///     // to correctly handle env-overrides,
    ///     // but this is too easy to miss.
    ///     // So we just don't implement `Default` at all.
    /// }, "0.0.0.0:3000").await;
    /// ```
    #[allow(unused)]
    pub(crate) const DEFAULT: Self = Self {
        #[cfg(feature = "__rt_native__")]
        request_bufsize: 1 << 11, // 2 KiB

        #[cfg(feature = "__rt_native__")]
        request_payload_limit: 1 << 32, // 4 GiB

        #[cfg(feature = "__rt_native__")]
        keepalive_timeout: 30, // 30 seconds

        #[cfg(feature = "__rt_native__")]
        #[cfg(feature = "ws")]
        websocket_timeout: 60 * 60, // 1 hour

        __private__: (),
    };

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "__rt_native__")]
            request_bufsize: parse_env("OHKAMI_REQUEST_BUFSIZE")
                .unwrap_or(Self::DEFAULT.request_bufsize),

            #[cfg(feature = "__rt_native__")]
            request_payload_limit: parse_env("OHKAMI_REQUEST_PAYLOAD_LIMIT")
                .unwrap_or(Self::DEFAULT.request_payload_limit),

            #[cfg(feature = "__rt_native__")]
            keepalive_timeout: parse_env("OHKAMI_KEEPALIVE_TIMEOUT")
                .unwrap_or(Self::DEFAULT.keepalive_timeout),

            #[cfg(feature = "__rt_native__")]
            #[cfg(feature = "ws")]
            websocket_timeout: parse_env("OHKAMI_WEBSOCKET_TIMEOUT")
                .unwrap_or(Self::DEFAULT.websocket_timeout),

            __private__: (),
        }
    }
}

#[allow(unused)]
fn parse_env<T: std::str::FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok().map(|val| {
        val.parse().unwrap_or_else(|err| {
            panic!(
                "failed to parse environment variable `{key}` as {}: `{val}`",
                std::any::type_name::<T>(),
            )
        })
    })
}
