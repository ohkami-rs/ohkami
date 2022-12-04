#[cfg(all(not(feature = "sqlx"), any(feature = "postgres", feature = "mysql")))]
compile_error!("feature `postgres` or `mysql` can't be enebled without enabling `sqlx` feature");
#[cfg(all(feature = "postgres", feature = "mysql"))]
compile_error!("`postgres` feature and `mysql` feature can't be enabled at the same time");

pub mod server;
pub mod result;
pub mod context;
pub mod response;
pub mod db;
pub mod components;
pub(crate) mod utils;

pub mod prelude {
    pub use super::{
        server::{Server, Config},
        result::{Result, ElseResponse, ElseResponseWithErr},
        context::Context,
        response::Response,
        components::json::JSON,
    };
    #[cfg(any(feature = "postgres", feature = "mysql"))]
    pub use super::db::useDB;
}