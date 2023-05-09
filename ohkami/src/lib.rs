/*===== global settings =====*/
#![doc(html_root_url = "https://docs.rs/ohkami/0.9.0")]

#![feature(try_trait_v2, byte_slice_trim_ascii)]

#![allow(incomplete_features)]
#![feature(adt_const_params, specialization)]


/*===== feature management =====*/
#[cfg(all(
    feature="async-std",
    feature="tokio",
))] compile_error!("any two of features

- `tokio`
- `async-std`

can be enabled at once.
");


/*===== modules =====*/
mod ohkami;
mod error;
mod context;
mod response;
mod request;
mod fang;
mod router;
mod handler;


/*===== public private =====*/
pub mod __ {
    pub use ohkami_macros::{json};
}


/*===== utils =====*/
pub mod utils {
    pub type Result<T> = std::result::Result<T, crate::error::Error>;
    
    #[macro_export]
    macro_rules! f {
        ($string:literal $(, $arg:expr)*) => {
            format!($string $(, $arg)*)
        };
        ({ $( $content:tt )+ }) => {
            ohkami::__::json!({ $( $content )+ })
        };
    }    
}


/*===== prelude =====*/
pub mod prelude {
    pub use super::{
        Error,
        Context,
        Response,
        utils::Result,
    };
}


/*===== in-crate reexport =====*/
pub(crate) use handler::{Handler};
pub(crate) use fang::FangRoutePattern;
pub(crate) use router::{Router, trie_tree::TrieTree};


/*===== public reexport =====*/
pub use ohkami::Ohkami;
pub use error::{Error, CatchError};
pub use context::Context;
pub use response::Response;
pub use request::{Request, from_request::FromRequest};
pub use fang::{Fang, Fangs, FangsRoute, IntoFang};
pub use handler::route::Route;
