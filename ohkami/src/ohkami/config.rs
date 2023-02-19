use async_std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::SubscriberBuilder;
use crate::error::Error;


#[cfg(feature="sqlx-postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature="sqlx-mysql")]
use sqlx::MySqlPool as ConnectionPool;
#[cfg(feature="deadpool-postgres")]
const _: (/* WIP */) = {};

pub static CONFIG: Config = Config(
    Arc::new(Mutex::new(OhkamiConfig {
        log_subscribe:
            Some(
                tracing_subscriber::fmt()
                    .with_max_level(tracing::Level::DEBUG)
            ),
        #[cfg(any(
            feature="sqlx-postgres",
            feature="sqlx-mysql",
            feature="deadpool-postgres",
        ))]
        connection_pool: None,
    }))
);

pub(crate) struct OhkamiConfig {
    pub(crate) log_subscribe: Option<SubscriberBuilder>,

    #[cfg(any(feature="sqlx-postgres", feature="sqlx-mysql", feature="deadpool-postgres"))]
    connection_pool: Option<ConnectionPool>,
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

        #[cfg(any(feature="sqlx-postgres", feature="sqlx-mysql"))]
        pub fn connection_pool(mut self, connection_pool: ConnectionPool) -> Self {
            todo!()
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
