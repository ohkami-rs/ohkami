use std::io::Error;
use super::{Message, Config};
use crate::__rt__::{AsyncWriter, AsyncReader};


/* Used only in `ohkami::ws::WebSocketContext::{connect, connect_with}` and NOT `use`able by user */

/// WebSocket connection
pub struct Connection<Conn: AsyncWriter + AsyncReader + Unpin + Send> {
    conn:       *mut Conn,
    config:     Config,
    n_buffered: usize,
}

const _: () = {
    unsafe impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Send for Connection<Conn> {}
    unsafe impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Sync for Connection<Conn> {}
    
    impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Connection<Conn> {
        /// SAFETY: `conn` is valid while entire the conversation
        pub(crate) unsafe fn new(conn: &mut Conn, config: Config) -> Self {
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

impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Connection<Conn> {
    /// Recieve a WebSocket message.
    /// 
    /// *note* : this automatically responds to a ping message
    pub async fn recv(&mut self) -> Result<Option<Message>, Error> {
        let message = Message::read_from(unsafe {&mut *self.conn}, &self.config).await?;
        if let Some(Message::Ping(payload)) = &message {
            self.send(Message::Pong(payload.clone())).await?;
        }
        Ok(message)
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


#[cfg(feature="rt_tokio")]
pub mod split {
    use super::*;
    use tokio::net::{TcpStream, tcp::{ReadHalf as TcpReadHalf, WriteHalf as TcpWriteHalf}};


    impl Connection<TcpStream> {
        pub fn split(self) -> (ReadHalf, WriteHalf) {
            let (read, write) = unsafe {self.conn.as_mut().unwrap()}.split();
            (
                ReadHalf  { conn: read,  config: self.config.clone() },
                WriteHalf { conn: write, config: self.config, n_buffered: self.n_buffered }
            )
        }
    }

    pub struct ReadHalf {
        conn:   TcpReadHalf<'static>,
        config: Config,
    }
    impl ReadHalf {
        /// Recieve a WebSocket message.
        /// 
        /// *note* : this **doesn't** automatically respond to ping messages
        pub async fn recv(&mut self) -> Result<Option<Message>, Error> {
            Message::read_from(&mut self.conn, &self.config).await
        }
    }

    pub struct WriteHalf {
        conn:       TcpWriteHalf<'static>,
        config:     Config,
        n_buffered: usize,
    }
    impl WriteHalf {
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
}
