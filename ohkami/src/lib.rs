/*===== feature managements =====*/
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


/*===== feature abstraction layer =====*/
mod __feature__ {
    #[cfg(feature="rt_tokio")]
    pub(crate) use tokio::net::TcpStream as TcpStream;
    #[cfg(feature="rt_async-std")]
    pub(crate) use async_std::net::TcpStream as TcpStream;
}


/*===== modules (the lower is the low-layer (: not-depending on other modules)) =====*/
mod handler;
mod fang;

mod context;

mod response;
mod request;


/*===== visibility managements =====*/
pub use context::Context;
// pub use response::;
