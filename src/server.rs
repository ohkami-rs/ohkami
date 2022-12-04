mod default;
mod postgres;
mod mysql;

#[cfg(not(any(feature = "postgres", feature = "mysql")))]
pub use default::server::{Server, ServerSetting, Config};

#[cfg(feature = "postgres")]
pub use postgres::server::{Server, ServerSetting, Config};

#[cfg(feature = "mysql")]
pub use mysql::server::{Server, ServerSetting, Config};