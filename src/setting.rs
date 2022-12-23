mod config;
mod middleware;

pub use config::Config;
pub use middleware::Middleware;
pub(crate) use middleware::MiddlewareFunc;

pub struct ServerSetting {
    pub(crate) config: Config,
    pub(crate) middleware: Middleware,
} impl ServerSetting {
    pub fn and(self, another: Self) -> Self {
        Self {
            config: another.config,
            middleware: Middleware::merge(self.middleware, another.middleware),
        }
    }
}
impl Default for ServerSetting {
    fn default() -> Self {
        Self {
            config: Config::default(),
            middleware: Middleware::init(),
        }
    }
}

pub trait IntoServerSetting {
    fn into_setting(self) -> ServerSetting;
    fn and<ISS: IntoServerSetting>(self, another: ISS) -> ServerSetting
    where
        Self: Sized
    {
        self.into_setting().and(another.into_setting())
    }
}
impl IntoServerSetting for Config {
    fn into_setting(self) -> ServerSetting {
        ServerSetting {
            config: self,
            middleware: Middleware::init(),
        }
    }
}
impl IntoServerSetting for Middleware {
    fn into_setting(self) -> ServerSetting {
        ServerSetting {
            config: Config::default(),
            middleware: self,
        }
    }
}