use async_std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::SubscriberBuilder;
use crate::error::Error;

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
    pub(crate) log_subscribe: Option<SubscriberBuilder>,

    #[cfg(any(feature="sqlx-postgres", feature="sqlx-mysql", feature="deadpool-postgres"))]
    connection_pool: ConnectionPool,
}
// impl Clone for OhkamiConfig {
//     fn clone(&self) -> Self {
//         let c = self.log_subscribe
//             .map(|l| l..clone());
//         todo!();
//         Self {
//             log_subscribe: ,
//         }
//     }
// }

pub struct Config(
    pub(crate) Arc<Mutex<OhkamiConfig>>
); pub const _: (/* Config impls */) = {
    impl Config {
        pub fn log_subscribe<LSC: LogSubscribeConfig>(mut self, log_subscribe_config: LSC) -> Self {
            self.0.get_mut().log_subscribe = log_subscribe_config.value();
            self
        }
    }

};
impl Config {
    pub(super) fn try_unwrap(self) -> crate::Result<OhkamiConfig> {
        Ok(
            Arc::try_unwrap(self.0)
                .map_err(|_| Error::Others(format!("Failed to get config")))?
                .into_inner()
        )
    }
}

trait LogSubscribeConfig {fn value(self) -> Option<SubscriberBuilder>;}
impl LogSubscribeConfig for Option<SubscriberBuilder> {fn value(self) -> Option<SubscriberBuilder> {self}}
impl LogSubscribeConfig for SubscriberBuilder {fn value(self) -> Option<SubscriberBuilder> {Some(self)}}
