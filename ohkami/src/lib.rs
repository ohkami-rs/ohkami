#![doc = include_str!("../../README.md")]
#![doc(html_root_url = "https://docs.rs/ohkami")]


/*===== crate features =====*/

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


/*===== async runtime dependency layer =====*/

mod __rt__ {
    #[allow(unused)]
    #[cfg(all(feature="rt_tokio", feature="DEBUG"))]
    pub(crate) use tokio::test;
    #[allow(unused)]
    #[cfg(all(feature="rt_async-std", feature="DEBUG"))]
    pub(crate) use async_std::test;

    #[cfg(all(feature="websocket", feature="rt_tokio"))]
    pub(crate) use tokio::net::TcpStream;
    #[cfg(all(feature="websocket", feature="rt_async-std"))]
    pub(crate) use async_std::net::TcpStream;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::TcpListener;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::TcpListener;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::sync::Mutex;
    #[allow(unused)]
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::sync::Mutex;

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


/*===== modules =====*/

mod layer0_lib;
mod layer1_req_res;
mod layer2_fang_handler;
mod layer3_router;
mod layer4_ohkami;

#[cfg(feature="testing")]
mod x_testing;

#[cfg(feature="utils")]
mod x_utils;

#[cfg(feature="websocket")]
mod x_websocket;


/*===== visibility managements =====*/

pub use layer1_req_res     ::{Request, Response, FromRequestError, FromRequest, FromParam, IntoResponse, Memory};
pub use layer2_fang_handler::{Route, Fang};
pub use layer4_ohkami      ::{Ohkami, IntoFang};

pub mod prelude {
    pub use crate::{Request, Route, Ohkami, Fang, Response, IntoFang, IntoResponse, http::Status};

    #[cfg(feature="utils")]
    pub use crate::typed::{OK, Created, NoContent};
}

pub mod http {
    pub use crate::layer0_lib::{Status, Method, append};
}

#[cfg(feature="testing")]
pub mod testing {
    pub use crate::x_testing::*;
}

#[cfg(feature="utils")]
pub mod utils {
    pub use crate::x_utils::{imf_fixdate_now, unix_timestamp, Text, HTML};
}

#[cfg(feature="utils")]
pub mod fangs {
    pub use crate::x_utils::{CORS, JWT};
}

#[cfg(feature="utils")]
pub mod serde {
    pub use ::ohkami_macros::{Serialize, Deserialize};
    pub use ::serde::ser::{self, Serialize, Serializer};
    pub use ::serde::de::{self, Deserialize, Deserializer};
}

#[cfg(feature="utils")]
pub mod typed {
    pub use ohkami_macros::{ResponseBody, Query, Payload};

    pub use crate::x_utils::{ResponseBody, File};

    pub use crate::x_utils::{
        SwitchingProtocols,

        OK,
        Created,
        NoContent,

        MovedPermanently,
        Found,

        BadRequest,
        Unauthorized,
        Forbidden,
        NotFound,
        UnprocessableEntity,

        InternalServerError,
        NotImplemented,
    };
}

#[cfg(feature="websocket")]
pub mod websocket {
    pub use crate::x_websocket::*;
}

#[doc(hidden)]
pub mod __internal__ {
    #[cfg(feature="utils")]
    pub use ::serde;

    #[cfg(feature="utils")]
    pub use ohkami_macros::consume_struct;

    #[cfg(feature="utils")]
    pub use crate::x_utils::{
        parse_json,
        parse_formparts,
        parse_urlencoded,
    };

    /* for benchmarks */
    #[cfg(feature="DEBUG")]
    pub use crate::layer1_req_res::{RequestHeader, RequestHeaders, ResponseHeader, ResponseHeaders};
}
