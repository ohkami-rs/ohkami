#![feature(try_trait_v2)]

#![allow(incomplete_features)]
#![feature(adt_const_params)]

#![doc(html_root_url = "https://docs.rs/ohkami/0.8.3")]

#[cfg(all(not(feature = "sqlx"), any(feature = "postgres", feature = "mysql")))]
compile_error!("feature `postgres` or `mysql` can't be enebled without enabling `sqlx` feature");
#[cfg(all(feature = "postgres", feature = "mysql"))]
compile_error!("`postgres` feature and `mysql` feature can't be enabled at the same time");

pub mod ohkami;
pub mod error;
pub mod context;
pub mod response;
pub mod request;
pub mod components;
// pub mod testing;
pub mod fang;
pub(crate) mod utils;
pub(crate) mod router;
pub(crate) mod handler;

pub type Result<T> = std::result::Result<T, error::Error>;

pub mod prelude {
    pub use super::{
        // ohkami::Ohkami,
        error::{ElseResponse, ElseResponseWithErr},
        context::Context,
        response::{Response},
    };
    pub use ohkami_macros::{JSON, json};

    #[cfg(feature = "sqlx")]
    pub use super::server::DBprofile;
}

pub mod macros {
    pub use ohkami_macros::{JSON, consume_struct, json};
}
