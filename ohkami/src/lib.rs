#![doc(html_root_url = "https://docs.rs/ohkami/0.6.6")]

#[cfg(all(not(feature = "sqlx"), any(feature = "postgres", feature = "mysql")))]
compile_error!("feature `postgres` or `mysql` can't be enebled without enabling `sqlx` feature");
#[cfg(all(feature = "postgres", feature = "mysql"))]
compile_error!("`postgres` feature and `mysql` feature can't be enabled at the same time");

pub mod server;
pub mod result;
pub mod context;
pub mod response;
pub mod components;
pub mod testing;
pub(crate) mod utils;
pub(crate) mod router;
pub(crate) mod handler;
pub(crate) mod setting;

pub mod prelude {
    pub use super::{
        server::Ohkami,
        setting::{Config, Middleware},
        result::{Result, ElseResponse, ElseResponseWithErr},
        context::Context,
        response::{Response, body::Body},
        components::headers::Header,
    };
    pub use ohkami_macros::JSON;

    #[cfg(feature = "sqlx")]
    pub use super::server::DBprofile;
}

pub use handler::group;

pub use ohkami_macros::JSON;