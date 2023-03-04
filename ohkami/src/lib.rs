#![feature(try_trait_v2, byte_slice_trim_ascii)]

#![allow(incomplete_features)]
#![feature(adt_const_params)]

#![doc(html_root_url = "https://docs.rs/ohkami/0.8.3")]

#[cfg(any(
    all(feature="sqlx-postgres", feature="sqlx-mysql"),
    all(feature="sqlx-postgres", feature="deadpool-postgres"),
    all(feature="sqlx-mysql", feature="deadpool-postgres"),
))]
compile_error!("any two of features

- sqlx-postgres
- sqlx-mysql
- deadpool-postgres

can be enabled at once.
");

pub mod ohkami;
pub mod error;
pub mod context;
pub mod response;
pub mod request;
// pub mod testing;
pub mod fang;
pub(crate) mod router;
pub(crate) mod handler;

pub type Result<T> = std::result::Result<T, error::Error>;

pub mod prelude {
    pub use super::{
        // ohkami::Ohkami,
        error::{ElseResponse, ElseResponseWithErr},
        context::Context,
        response::Response,
    };
    pub use ohkami_macros::{JSON, json};
}

pub mod macros {
    pub use ohkami_macros::{JSON, consume_struct, json};
}
