/*===== language features =====*/
#![feature(
    try_trait_v2,
    fn_traits, unboxed_closures,
)]


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


/*===== dependency injection layer =====*/
mod __dep__ {
    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::TcpStream as TcpStream;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::TcpStream as TcpStream;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::__TODO__ as StreamReader;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::ReadExt as StreamReader;

    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::__TODO__ as StreamWriter;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::io::WriteExt as StreamWriter;
}


/*===== modules =====*/
mod layer0_lib;
mod layer1_req_res;
mod layer2_context;
mod layer3_fang_handler;
mod layer4_router;
mod layer5_ohkami;


/*===== visibility managements =====*/
pub(crate) use layer1_req_res::{QUERIES_LIMIT, HEADERS_LIMIT};
pub(crate) use layer3_fang_handler::{PATH_PARAMS_LIMIT};

pub use layer0_lib::{Error};
pub use layer1_req_res::{Request, Response};
pub use layer2_context::{Context};
