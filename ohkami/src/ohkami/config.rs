use async_std::sync::{Arc, Mutex};
use tracing_subscriber::FmtSubscriber;

pub static CONFIG: Config = Config(
    Arc::new(Mutex::new(OhkamiConfig {
        log_subscribe:
            Some(
                tracing_subscriber::fmt()
                    .with_max_level(tracing::Level::DEBUG)
            ),
    }))
);

pub(crate) struct OhkamiConfig {
    pub(crate) log_subscribe: Option<FmtSubscriber>,

    #[cfg(any(feature="sqlx-postgres", feature="sqlx-mysql", feature="deadpool-postgres"))]
    connection_pool: ConnectionPool,
}




pub struct Config(
    pub(crate) Arc<Mutex<OhkamiConfig>>
); const _: (/* Config impls */) = {
    impl Config {
        pub fn log_subscribe<LSC: LogSubscribeConfig>(mut self, log_subscribe_config: LSC) -> Self {
            self.0.get_mut().log_subscribe = log_subscribe_config.value();
            self
        }
    }
};

trait LogSubscribeConfig {fn value(self) -> Option<FmtSubscriber>;}
impl LogSubscribeConfig for Option<FmtSubscriber> {fn value(self) -> Option<FmtSubscriber> {self}}
impl LogSubscribeConfig for FmtSubscriber {fn value(self) -> Option<FmtSubscriber> {Some(self)}}
