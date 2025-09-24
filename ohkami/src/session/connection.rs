pub enum Connection {
    Tcp(crate::__rt__::TcpStream),
    #[cfg(feature = "tls")]
    Tls(anysc_rustls::server::TlsStream<crate::__rt__::TcpStream>),
}

impl From<crate::__rt__::TcpStream> for Connection {
    #[inline]
    fn from(stream: crate::__rt__::TcpStream) -> Self {
        Self::Tcp(stream)
    }
}
#[cfg(feature = "tls")]
impl From<anysc_rustls::server::TlsStream<crate::__rt__::TcpStream>> for Connection {
    #[inline]
    fn from(stream: anysc_rustls::server::TlsStream<crate::__rt__::TcpStream>) -> Self {
        Self::Tls(stream)
    }
}

#[cfg(feature = "__io_tokio__")]
const _: () = {
    impl tokio::io::AsyncRead for Connection {
        #[inline(always)]
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
                #[cfg(feature = "tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            }
        }
    }

    impl tokio::io::AsyncWrite for Connection {
        #[inline(always)]
        fn poll_write(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
                #[cfg(feature = "tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            }
        }

        #[inline]
        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_flush(cx),
                #[cfg(feature = "tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            }
        }

        fn poll_shutdown(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
                #[cfg(feature = "tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_shutdown(cx),
            }
        }
    }
};

#[cfg(feature = "__io_futures__")]
const _: () = {
    impl futures_util::io::AsyncRead for Connection {
        #[inline(always)]
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut [u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
                #[cfg(feature = "tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_read(cx, buf),
            }
        }
    }

    impl futures_util::io::AsyncWrite for Connection {
        #[inline(always)]
        fn poll_write(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
                #[cfg(feature = "tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_write(cx, buf),
            }
        }

        #[inline]
        fn poll_flush(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_flush(cx),
                #[cfg(feature = "tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_flush(cx),
            }
        }

        fn poll_close(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            match std::pin::Pin::into_inner(self) {
                Self::Tcp(stream) => std::pin::Pin::new(stream).poll_close(cx),
                #[cfg(feature = "tls")]
                Self::Tls(stream) => std::pin::Pin::new(stream).poll_close(cx),
            }
        }
    }
};
