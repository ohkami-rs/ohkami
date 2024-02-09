#![doc(html_root_url = "https://docs.rs/ohkami")]

/* Execute static tests for sample codes in README */
#![cfg_attr(feature="DEBUG", doc = include_str!("../../README.md"))]

//! <div align="center">
//!     <h1>ohkami</h1>
//!     ohkami <em>- [狼] wolf in Japanese -</em> is intuitive and declarative web framework.
//! </div>
//! 
//! - *intuitive and declarative* APIs
//! - *multi runtime* support：`tokio`, `async-std`
//! 
//! See our [README](https://github.com/kana-rus/ohkami/blob/main/README.md)
//! and [examples](https://github.com/kana-rus/ohkami/tree/main/examples)
//! for more information！


#[cfg(any(
    all(feature="rt_tokio", feature="rt_async-std")
))] compile_error!("
    Can't activate multiple `rt_*` feature!
");

#[cfg(not(any(
    feature="rt_tokio",
    feature="rt_async-std",
)))] compile_error!("
    Activate 1 of `rt_*` features：
    - rt_tokio
    - rt_async-std
");


mod __rt__ {
    #[allow(unused)]
    #[cfg(all(feature="rt_tokio", feature="DEBUG"))]
    pub(crate) use tokio::test;
    #[allow(unused)]
    #[cfg(all(feature="rt_async-std", feature="DEBUG"))]
    pub(crate) use async_std::test;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::{TcpListener, TcpStream, ToSocketAddrs};

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::task;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::task;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncReadExt as AsyncReader;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::ReadExt as AsyncReader;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::io::AsyncWriteExt as AsyncWriter;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::WriteExt as AsyncWriter;

    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::stream::StreamExt;
}


mod request;
pub use request::{Request, Method, FromRequestError, FromRequest, FromParam, Memory};

mod response;
pub use response::{Response, Status, IntoResponse};

mod handler;
pub use handler::Route;

mod fang;
pub use fang::{Fang, builtin};

mod session;
use session::Session;

mod ohkami;
pub use ohkami::{Ohkami, IntoFang};

pub mod typed;

#[cfg(feature="testing")]
pub mod testing;

#[cfg(feature="utils")]
pub mod utils;

#[cfg(feature="websocket")]
mod x_websocket;


/// Passed to `{Request/Response}.headers.set().Name( 〜 )` and
/// append `value` to the header
/// 
/// <br>
/// 
/// *example.rs*
/// ```
/// # use ohkami::prelude::*;
/// use ohkami::append;
/// 
/// struct AppendServer;
/// impl IntoFang for AppendServer {
///     fn into_fang(self) -> Fang {
///         Fang::back(|res: &mut Response| {
///             res.headers.set()
///                 .Server(append("ohkami"));
///         })
///     }
/// }
/// ```
pub fn append(value: impl Into<std::borrow::Cow<'static, str>>) -> __internal__::Append {
    __internal__::Append(value.into())
}

pub mod prelude {
    pub use crate::{Request, Route, Ohkami, Fang, Response, IntoFang, IntoResponse, Method, Status};
}

/// Somthing that's almost [serde](https://crates.io/crates/serde)
/// 
/// <br>
/// 
/// *not_need_serde_in_your_dependencies.rs*
/// ```
/// use ohkami::serde::Serialize;
/// 
/// #[derive(Serialize)]
/// struct User {
///     #[serde(rename = "username")]
///     name: String,
///     age:  u8,
/// }
/// ```
pub mod serde {
    pub use ::ohkami_macros::{Serialize, Deserialize};
    pub use ::serde::ser::{self, Serialize, Serializer};
    pub use ::serde::de::{self, Deserialize, Deserializer};
}

#[cfg(feature="websocket")]
pub mod websocket {
    pub use crate::x_websocket::*;
}

#[doc(hidden)]
pub mod __internal__ {
    pub struct Append(pub(crate) std::borrow::Cow<'static, str>);

    pub use ::serde;

    pub use ohkami_macros::consume_struct;

    pub use crate::typed::parse_payload::{
        parse_json,
        parse_formparts,
        parse_urlencoded,
    };

    /* for benchmarks */
    #[cfg(feature="DEBUG")]
    pub use crate::{
        request::{RequestHeader, RequestHeaders},
        response::{ResponseHeader, ResponseHeaders},
    };
}
