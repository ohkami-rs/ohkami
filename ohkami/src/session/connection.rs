#[derive(Debug)]
pub enum Connection {
    Tcp(crate::__rt__::TcpStream),
    #[cfg(feature="tls")]
    Tls(crate::tls::TlsStream),
}

impl From<crate::__rt__::TcpStream> for Connection {
    fn from(stream: crate::__rt__::TcpStream) -> Self {
        Self::Tcp(stream)
    }
}
#[cfg(feature="tls")]
impl From<crate::tls::TlsStream> for Connection {
    fn from(stream: crate::tls::TlsStream) -> Self {
        Self::Tls(stream)
    }
}

#[cfg(feature="rt_tokio")]
const _: () = {
    impl tokio::io::AsyncRead for Connection {
        fn poll_read(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>, 
            buf: &mut tokio::io::ReadBuf<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
                #[cfg(feature="tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            }
        }
    }
    
    impl tokio::io::AsyncWrite for Connection {
        fn poll_write(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>, 
            buf: &[u8]
        ) -> std::task::Poll<std::io::Result<usize>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
                #[cfg(feature="tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            }
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_flush(cx),
                #[cfg(feature="tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            }
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
                #[cfg(feature="tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            }
        }
    }
    
    #[cfg(feature="ws")]
    #[cfg(not(feature="tls"))]
    impl<'split> mews::Splitable<'split> for Connection {
        type ReadHalf = <crate::__rt__::TcpStream as mews::Splitable<'split>>::ReadHalf;
        type WriteHalf = <crate::__rt__::TcpStream as mews::Splitable<'split>>::WriteHalf;
        fn split(&'split mut self) -> (Self::ReadHalf, Self::WriteHalf) {
            match self {
                Self::Tcp(stream) => stream.split(),
            }
        }
    }
    #[cfg(feature="ws")]
    #[cfg(feature="tls")]
    impl<'split> mews::Splitable<'split> for Connection {
        // generic split impl, maybe a few less efficient than simple `TcpStream` split
        type ReadHalf = TokioIoReadHalf<'split, Connection>;
        type WriteHalf = TokioIoWriteHalf<'split, Connection>;
        fn split(&'split mut self) -> (Self::ReadHalf, Self::WriteHalf) {
            let (r, w) = futures_util::lock::BiLock::new(self);
            (TokioIoReadHalf(r), TokioIoWriteHalf(w))
        }
    }
    /* based on https://github.com/rust-lang/futures-rs/blob/de9274e655b2fff8c9630a259a473b71a6b79dda/futures-util/src/io/split.rs */    
    #[cfg(all(feature="ws", feature="tls"))]
    pub struct TokioIoReadHalf<'split, T>(
        futures_util::lock::BiLock<&'split mut T>
    );
    #[cfg(all(feature="ws", feature="tls"))]
    pub struct TokioIoWriteHalf<'split, T>(
        futures_util::lock::BiLock<&'split mut T>
    );
    #[cfg(all(feature="ws", feature="tls"))]
    fn lock_and_then<T, U, E>(
        lock: &futures_util::lock::BiLock<T>,
        cx: &mut std::task::Context<'_>,
        f: impl FnOnce(std::pin::Pin<&mut T>, &mut std::task::Context<'_>) -> std::task::Poll<Result<U, E>>
    ) -> std::task::Poll<Result<U, E>> {
        let mut l = futures_util::ready!(lock.poll_lock(cx));
        f(l.as_pin_mut(), cx)
    }
    #[cfg(all(feature="ws", feature="tls"))]
    impl<'split, T: tokio::io::AsyncRead + Unpin> tokio::io::AsyncRead for TokioIoReadHalf<'split, T> {
        fn poll_read(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>, 
            buf: &mut tokio::io::ReadBuf<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            lock_and_then(&self.0, cx, |l, cx| l.poll_read(cx, buf))
        }
    }
    #[cfg(all(feature="ws", feature="tls"))]
    impl<'split, T: tokio::io::AsyncWrite + Unpin> tokio::io::AsyncWrite for TokioIoWriteHalf<'split, T> {
        fn poll_write(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>, 
            buf: &[u8]
        ) -> std::task::Poll<std::io::Result<usize>> {
            lock_and_then(&self.0, cx, |l, cx| l.poll_write(cx, buf))
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            lock_and_then(&self.0, cx, |l, cx| l.poll_flush(cx))
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            lock_and_then(&self.0, cx, |l, cx| l.poll_shutdown(cx))
        }
    }
};

/*
 * Currently `tls` feature is only supported on `rt_tokio`.
 */

#[cfg(feature="rt_smol")]
const _: () = {
    impl smol::io::AsyncRead for Connection {
        fn poll_read(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>, 
            buf: &mut smol::io::ReadBuf<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            }
        }
    }
    
    impl smol::io::AsyncWrite for Connection {
        fn poll_write(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>, 
            buf: &[u8]
        ) -> std::task::Poll<std::io::Result<usize>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            }
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            }
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            }
        }
    }
    
    #[cfg(feature="ws")]
    impl<'split> mews::Splitable<'split> for Connection {
        type ReadHalf = <crate::__rt__::TcpStream as mews::Splitable>::ReadHalf;
        type WriteHalf = <crate::__rt__::TcpStream as mews::Splitable>::WriteHalf;
        fn split(&'split mut self) -> (Self::ReadHalf, Self::WriteHalf) {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => stream.split(),
            }
        }
    }
};

#[cfg(feature="rt_glommio")]
const _: () = {
    impl futures_util::AsyncRead for Connection {
        fn poll_read(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>, 
            buf: &mut glommio::io::ReadBuf<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            }
        }
    }
    
    impl futures_util::AsyncWrite for Connection {
        fn poll_write(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>, 
            buf: &[u8]
        ) -> std::task::Poll<std::io::Result<usize>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            }
        }

        fn poll_flush(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            }
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>, 
            cx: &mut std::task::Context<'_>
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            }
        }
    }
    
    #[cfg(feature="ws")]
    impl<'split> mews::Splitable<'split> for Connection {
        type ReadHalf = <crate::__rt__::TcpStream as mews::Splitable>::ReadHalf;
        type WriteHalf = <crate::__rt__::TcpStream as mews::Splitable>::WriteHalf;
        fn split(&'split mut self) -> (Self::ReadHalf, Self::WriteHalf) {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => stream.split(),
            }
        }
    }
};
