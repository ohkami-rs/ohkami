mod config;
mod middleware;

pub use config::Config;
pub use middleware::Middleware;
pub(crate) use middleware::{
    BeforeMiddleware, BeforeMiddlewareStore,
    AfterMiddleware, AfterMiddlewareStore,
};

#[cfg(not(feature = "sqlx"))]
pub struct ServerSetting {
    pub(crate) config: Config,
    pub(crate) middleware: Middleware,
}
#[cfg(not(feature = "sqlx"))]
impl ServerSetting {
    pub fn and(self, another: Self) -> Self {
        Self {
            config: another.config,
            middleware: Middleware::merge(self.middleware, another.middleware),
        }
    }
}

#[cfg(feature = "sqlx")]
pub struct ServerSetting<'url> {
    pub(crate) config: Config<'url>,
    pub(crate) middleware: Middleware,
}
#[cfg(feature = "sqlx")]
impl ServerSetting<'_> {
    pub fn and(self, another: Self) -> Self {
        Self {
            config: another.config,
            middleware: Middleware::merge(self.middleware, another.middleware),
        }
    }
}

#[cfg(not(feature = "sqlx"))]
impl Default for ServerSetting {
    fn default() -> Self {
        Self {
            config: Config::default(),
            middleware: Middleware::new(),
        }
    }
}
#[cfg(feature = "sqlx")]
impl Default for ServerSetting<'_> {
    fn default() -> Self {
        Self {
            config: Config::default(),
            middleware: Middleware::new(),
        }
    }
}


#[cfg(not(feature = "sqlx"))]
pub trait IntoServerSetting {
    fn into_setting(self) -> ServerSetting;
    fn and<ISS: IntoServerSetting>(self, another: ISS) -> ServerSetting where Self: Sized {
        self.into_setting().and(another.into_setting())
    }
}
#[cfg(feature = "sqlx")]
pub trait IntoServerSetting<'url> {
    fn into_setting(self) -> ServerSetting<'url>;
    fn and<ISS: IntoServerSetting<'url>>(self, another: ISS) -> ServerSetting<'url> where Self: Sized {
        self.into_setting().and(another.into_setting())
    }
}

#[cfg(not(feature = "sqlx"))]
impl IntoServerSetting for Middleware {
    fn into_setting(self) -> ServerSetting {
        ServerSetting {
            config: Config::default(),
            middleware: self,
        }
    }
}
#[cfg(feature = "sqlx")]
impl<'url> IntoServerSetting<'url> for Middleware {
    fn into_setting(self) -> ServerSetting<'url> {
        ServerSetting {
            config: Config::default(),
            middleware: self,
        }
    }
}

#[cfg(not(feature = "sqlx"))]
impl IntoServerSetting for Config {
    fn into_setting(self) -> ServerSetting {
        ServerSetting {
            config: self,
            middleware: Middleware::new(),
        }
    }
}
#[cfg(feature = "sqlx")]
impl<'url> IntoServerSetting<'url> for Config<'url> {
    fn into_setting(self) -> ServerSetting<'url> {
        ServerSetting {
            config: self,
            middleware: Middleware::new(),
        }
    }
}