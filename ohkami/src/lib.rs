#![feature(try_trait_v2, byte_slice_trim_ascii)]

#![allow(incomplete_features)]
#![feature(adt_const_params, specialization)]

#![doc(html_root_url = "https://docs.rs/ohkami/0.9.0")]

#[cfg(all(
    feature="async-std",
    feature="tokio",
    feature="lunatic",
))] compile_error!("any two of features

- `tokio`
- `async-std`
- `lunatic`

can be enabled at once.
");

mod ohkami;
pub use ohkami::Ohkami;

mod error;
pub use error::{Error, CatchError};

mod context;
pub use context::Context;

mod response;
pub use response::Response;

mod request;
pub use request::{Request, from_request::FromRequest};

mod fang;
pub use fang::{Fang, Fangs, FangsRoute, IntoFang};
pub(crate) use fang::FangRoutePattern;

mod router;
pub(crate) use router::{Router, trie_tree::TrieTree};

mod handler;
pub use handler::route::Route;
pub(crate) use handler::{Handler};

pub type Result<T> = std::result::Result<T, error::Error>;

pub mod prelude {
    pub use super::{
        Error,
        Context,
        Response,
        Result,
    };
}

pub mod utils {
    pub use ohkami_lib::{f};
}

pub mod __ {
    pub use ohkami_macros::{json};
}
