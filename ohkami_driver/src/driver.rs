#![cfg(any(feature = "__tokio_io__", feature = "__futures_io__"))]

#[cfg(feature = "__tokio_io__")]
pub use ::tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};

#[cfg(feature = "__futures_io__")]
pub use ::futures_util::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};

pub trait Driver {
    type TcpListener: TcpListener<Self::TcpStream>;
    type TcpStream: TcpStream;
    
    fn sleep(duration: std::time::Duration) -> impl Future<Output = ()> + crate::SendOnThreaded;
    fn spawn(
        task: impl Future<Output: crate::SendOnThreaded + 'static> + crate::SendOnThreaded + 'static
    );
}

pub trait TcpListener<S: TcpStream> {
    fn bind(address: std::net::SocketAddr) -> impl Future<Output = S> + crate::SendOnThreaded;
    fn accept(&self) -> impl Future<Output = (S, std::net::SocketAddr)> + crate::SendOnThreaded;
}

pub trait TcpStream: AsyncRead + AsyncWrite {
}
