#![cfg(feature="tls")]

use tokio::io::{AsyncRead, AsyncWrite};

pub struct TlsStream(pub(crate) anysc_rustls::server::TlsStream<crate::__rt__::TcpStream>);

impl std::fmt::Debug for TlsStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsyncRead for TlsStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>, 
        cx: &mut std::task::Context<'_>, 
        buf: &mut tokio::io::ReadBuf<'_>
    ) -> std::task::Poll<std::io::Result<()>> {
        match std::pin::Pin::new(&mut self.0).poll_read(cx, buf) {
            std::task::Poll::Ready(Err(e)) => {
                if e.to_string().contains("close_notify") {
                    std::task::Poll::Ready(Ok(()))
                } else {
                    std::task::Poll::Ready(Err(e))
                }
            },
            other => other,
        }
    }
}

impl AsyncWrite for TlsStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>, 
        cx: &mut std::task::Context<'_>, 
        buf: &[u8]
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>, 
        cx: &mut std::task::Context<'_>
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>, 
        cx: &mut std::task::Context<'_>
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_shutdown(cx)
    }
}