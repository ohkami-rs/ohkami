use tokio::io::{AsyncRead, AsyncWrite};

pub struct TlsStream(pub tokio_rustls::server::TlsStream<tokio::net::TcpStream>);

impl crate::session::Connection for TlsStream {
    #[cfg(feature="ws")]
    fn into_websocket_stream(self) -> Result<crate::__rt__::TcpStream, &'static str> {
        Err("WebSocket connections are not supported over TLS yet")
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