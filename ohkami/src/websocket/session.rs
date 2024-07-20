use std::io::Error;
use super::{Message, Config};
use crate::__rt__::{AsyncWriter, AsyncReader};


/* Used only in `ohkami::websocket::WebSocket::{new, with}` and NOT `use`able by user */

/// WebSocket connection
pub struct WebSocket<Conn: AsyncWriter + AsyncReader + Unpin + Send> {
    conn:       *mut Conn,
    config:     Config,
    n_buffered: usize,
}

const _: () = {
    unsafe impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Send for WebSocket<Conn> {}
    unsafe impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Sync for WebSocket<Conn> {}
    
    impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> WebSocket<Conn> {
        /// SAFETY: `conn` is valid while entire the conversation
        pub(crate) unsafe fn new(conn: &mut Conn, config: Config) -> Self {
            #[cfg(feature="DEBUG")] {
                println!("`websocket::session::WebSocket::new` called")
            }

            Self { conn, config, n_buffered:0 }
        }
    }
};

// =============================================================================
pub(super) async fn send(
    message:    Message,
    stream:     &mut (impl AsyncWriter + Unpin),
    config:     &Config,
    n_buffered: &mut usize,
) -> Result<(), Error> {
    message.write(stream, config).await?;
    flush(stream, n_buffered).await?;
    Ok(())
}
pub(super) async fn write(
    message:    Message,
    stream:     &mut (impl AsyncWriter + Unpin),
    config:     &Config,
    n_buffered: &mut usize,
) -> Result<usize, Error> {
    let n = message.write(stream, config).await?;

    *n_buffered += n;
    if *n_buffered > config.write_buffer_size {
        if *n_buffered > config.max_write_buffer_size {
            panic!("Buffered messages is larger than `max_write_buffer_size`");
        } else {
            flush(stream, n_buffered).await?
        }
    }

    Ok(n)
}
pub(super) async fn flush(
    stream:     &mut (impl AsyncWriter + Unpin),
    n_buffered: &mut usize,
) -> Result<(), Error> {
    stream.flush().await
        .map(|_| *n_buffered = 0)
}
// =============================================================================

impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> WebSocket<Conn> {
    pub async fn recv(&mut self) -> Result<Option<Message>, Error> {
        Message::read_from(unsafe {&mut *self.conn}, &self.config).await
    }

    pub async fn send(&mut self, message: Message) -> Result<(), Error> {
        send(message, unsafe {&mut *self.conn}, &self.config, &mut self.n_buffered).await
    }

    pub async fn write(&mut self, message: Message) -> Result<usize, Error> {
        write(message, unsafe {&mut *self.conn}, &self.config, &mut self.n_buffered).await
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        flush(unsafe {&mut *self.conn}, &mut self.n_buffered).await
    }
}
