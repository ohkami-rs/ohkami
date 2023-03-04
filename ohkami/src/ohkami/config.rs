use async_std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::SubscriberBuilder;


pub static CONFIG: Config = Config(
    Arc::new(Mutex::new(OhkamiConfig {
        log_init: false,
    }))
);

pub(crate) struct OhkamiConfig {
    pub(crate) log_init: bool,
}

pub struct Config(
    pub(crate) Arc<Mutex<OhkamiConfig>>
);
impl Config {
    pub fn log_subscribe(mut self, log_subscribe_config: SubscriberBuilder) -> Self {
        let mut config = self.0.get_mut();
        if !(config.log_init) {
            log_subscribe_config.init();
            config.log_init = true;
        }
        self
    }
}
