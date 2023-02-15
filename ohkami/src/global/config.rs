use std::{cell::UnsafeCell, panic::{RefUnwindSafe, UnwindSafe}};
use tracing_subscriber::FmtSubscriber;
use crate::components::headers::ResponseHeaders;

pub struct Config(UnsafeCell<OhkamiConfig>);
pub struct OhkamiConfig {
    log_subscribe:   Option<FmtSubscriber>,
    default_headers: ResponseHeaders,

    #[cfg(any(feature="sqlx-postgres", feature="sqlx-mysql", feature="deadpool-postgres"))]
    connection_pool: ConnectionPool,
}

const _: (/* OhkamiConfig impls */) = {
    impl Default for OhkamiConfig {
        fn default() -> Self {
            Self {
                log_subscribe:   None,
                default_headers: ResponseHeaders::from(),
            }
        }
    }
};

impl<T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for Config {}
impl<T: UnwindSafe> UnwindSafe for Config {}

impl Default for Config {
    fn default() -> Self {
        Self(UnsafeCell::new(OhkamiConfig::default()))
    }
}



