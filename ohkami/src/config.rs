/// Configuration for Ohkami server.
/// 
/// This configuration can be installed only once by [`Config::install`] or [`Config::install_or_env`].
/// If not installed, the default configuration will be used.
/// Each field can be overridden by the corresponding environment variable.
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
    pub websocket_timeout: u64,

    #[doc(hidden)]
    pub __private__: (),
}

impl Default for Config {
    fn default() -> Self {
        Self {
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
        }
    }
}

static INSTALLER: std::sync::OnceLock<Installer> = std::sync::OnceLock::new();

#[allow(unused)]
struct Installer {
    config: Config,
    allow_env: bool,
}

/*
 * MEMO:
 * 
 * - This system relying on 2 static values seems a bit hacky, too **complicated**
 * - `.install()` and `.install_or_env()` is too **imperative** for Ohkami's design philosophy
 * 
 * If a more declarative design is possible, it will automatically make
 * the environment variable override more natural, and
 * remove the need for `install_or_env()` and `INSTALLER`.
 * 
 * - Pass `Config` to `Ohkami` instance directly?
 * - Use `OnceCell<Config>` and `get_or_init()` in `CONFIG` directly?
 */
impl Config {
    /// Install the configuration.
    /// This must be called only once, and before any server is started.
    pub fn install(self) {
        if INSTALLER
            .set(Installer {
                config: self,
                allow_env: false,
            })
            .is_err()
        {
            panic!("Config has already been installed");
        }
    }

    /// Install the configuration, but allow **environment variables to override values**.
    /// This must be called only once, and before any server is started.
    pub fn install_or_env(self) {
        if INSTALLER
            .set(Installer {
                config: self,
                allow_env: true,
            })
            .is_err()
        {
            panic!("Config has already been installed");
        }
    }
}

#[cfg(feature = "__rt_native__")]
pub(crate) static CONFIG: std::sync::LazyLock<Config> = std::sync::LazyLock::new(|| {
    #[allow(unused)]
    fn parse_env<T: std::str::FromStr>(key: &str) -> Option<T> {
        std::env::var(key).ok().and_then(|val| {
            val.parse()
                .inspect_err(|err| {
                    crate::WARNING!(
                        "failed to parse environment variable `{key}` as {}: `{val}`",
                        std::any::type_name::<T>(),
                    )
                })
                .ok()
        })
    }

    #[allow(unused)]
    let Installer { config, allow_env } = INSTALLER.get_or_init(|| Installer {
        config: Config::default(),
        allow_env: true,
    });

    #[allow(unused)]
    macro_rules! load {
        ($name:ident, $env:literal) => {
            parse_env($env)
                .filter(|_| *allow_env)
                .unwrap_or(config.$name)
        };
    }

    Config {
        #[cfg(feature = "__rt_native__")]
        request_bufsize: load!(request_bufsize, "OHKAMI_REQUEST_BUFSIZE"),

        #[cfg(feature = "__rt_native__")]
        request_payload_limit: load!(request_payload_limit, "OHKAMI_REQUEST_PAYLOAD_LIMIT"),

        #[cfg(feature = "__rt_native__")]
        keepalive_timeout: load!(keepalive_timeout, "OHKAMI_KEEPALIVE_TIMEOUT"),

        #[cfg(feature = "__rt_native__")]
        #[cfg(feature = "ws")]
        websocket_timeout: load!(websocket_timeout, "OHKAMI_WEBSOCKET_TIMEOUT"),

        __private__: (),
    }
});
