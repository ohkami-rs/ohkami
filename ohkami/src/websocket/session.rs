use std::io::Error;
use super::{Message, Config};
use crate::__rt__::{AsyncWriter, AsyncReader};


/* Used only in `ohkami::websocket::WebSocket::{new, with}` and NOT `use`able by user */
pub struct WebSocket<'ws, Conn: AsyncWriter + AsyncReader + Unpin> {
    conn:   &'ws mut Conn,
    config: Config,
    n_buffered: usize,
}

impl<'ws, Conn: AsyncWriter + AsyncReader + Unpin> WebSocket<'ws, Conn> {
    pub(crate) fn new(conn: &'ws mut Conn, config: Config) -> Self {
        Self { conn, config, n_buffered:0 }
    }
}

impl<'ws, Conn: AsyncWriter + AsyncReader + Unpin> WebSocket<'ws, Conn> {
    pub async fn recv(&mut self) -> Result<Option<Message>, Error> {
        Message::read_from(self.conn, &self.config).await
    }
}

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

impl<'ws, Conn: AsyncWriter + AsyncReader + Unpin> WebSocket<'ws, Conn> {
    pub async fn send(&mut self, message: Message) -> Result<(), Error> {
        send(message, &mut self.conn, &self.config, &mut self.n_buffered).await
    }

    pub async fn write(&mut self, message: Message) -> Result<usize, Error> {
        write(message, &mut self.conn, &self.config, &mut self.n_buffered).await
    }
    
    pub async fn flush(&mut self) -> Result<(), Error> {
        flush(&mut self.conn, &mut self.n_buffered).await
    }
}
