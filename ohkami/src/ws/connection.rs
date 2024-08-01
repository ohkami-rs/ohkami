use std::io::Error;
use std::{sync::Arc, cell::UnsafeCell};
use super::{Message, Config};
use crate::__rt__::{AsyncWriter, AsyncReader};


/// WebSocket connection
pub struct Connection<Conn: AsyncWriter + AsyncReader + Unpin + Send> {
    conn:       Arc<UnsafeCell<(State, Conn)>>,
    config:     Config,
    n_buffered: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum State { Alive, Closed }

const _: () = {
    unsafe impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Send for Connection<Conn> {}
    unsafe impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Sync for Connection<Conn> {}

    impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> std::fmt::Debug for Connection<Conn> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let (state, underlying) = unsafe {&*self.conn.get()};
            f.debug_struct("Connection")
                .field("underlying", &(underlying as *const _))
                .field("state", state)
                .field("config", &self.config)
                .field("n_buffered", &self.n_buffered)
                .finish()
        }
    }
    impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Clone for Connection<Conn> {
        fn clone(&self) -> Self {
            Connection {
                conn:       Arc::clone(&self.conn),
                config:     self.config.clone(),
                n_buffered: self.n_buffered
            }
        }
    }
    
    impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Connection<Conn> {
        pub(crate) fn new(conn: Conn, config: Config) -> Self {
            let conn = Arc::new(UnsafeCell::new((State::Alive, conn)));
            Self { conn, config, n_buffered:0 }
        }

        pub(crate) fn is_closed(&self) -> bool {
            unsafe {&*self.conn.get()}.0 == State::Closed
        }
    }

    impl State {
        fn assert_alive(&self) {
            (*self == State::Alive).then_some(()).expect("\n\
                |---------------------------------------------------------------------\n\
                | ws::Connection is already closed                                   |\n\
                |                                                                    |\n\
                | Maybe you spawned tasks using ws::Connection or split halves of it |\n\
                | and NOT join/await the tasks?                                      |\n\
                | This is NOT supported because it may cause resource leak           |\n\
                | due to an infinite loop in the websocket handler.                  |\n\
                | If you're doing it, please join/await the tasks in the handler!    |\n\
                ---------------------------------------------------------------------|\n\
            ")
        }
    }
};

// =============================================================================
pub(super) async fn send(
    message:    Message,
    conn:       &mut (impl AsyncWriter + Unpin),
    config:     &Config,
    n_buffered: &mut usize,
) -> Result<(), Error> {
    message.write(conn, config).await?;
    flush(conn, n_buffered).await?;
    Ok(())
}
pub(super) async fn write(
    message:    Message,
    conn:       &mut (impl AsyncWriter + Unpin),
    config:     &Config,
    n_buffered: &mut usize,
) -> Result<usize, Error> {
    let n = message.write(conn, config).await?;

    *n_buffered += n;
    if *n_buffered > config.write_buffer_size {
        if *n_buffered > config.max_write_buffer_size {
            panic!("Buffered messages is larger than `max_write_buffer_size`");
        } else {
            flush(conn, n_buffered).await?
        }
    }

    Ok(n)
}
pub(super) async fn flush(
    conn:       &mut (impl AsyncWriter + Unpin),
    n_buffered: &mut usize,
) -> Result<(), Error> {
    conn.flush().await
        .map(|_| *n_buffered = 0)
}
// =============================================================================

impl<Conn: AsyncWriter + AsyncReader + Unpin + Send> Connection<Conn> {
    /// Recieve a WebSocket message.
    /// 
    /// *note* : this automatically responds to a ping message
    pub async fn recv(&mut self) -> Result<Option<Message>, Error> {
        let (state, conn) = unsafe {&mut *self.conn.get()};
        state.assert_alive();

        let message = Message::read_from(conn, &self.config).await?;
        if let Some(Message::Ping(payload)) = &message {
            self.send(Message::Pong(payload.clone())).await?;
        }
        Ok(message)
    }

    pub async fn send(&mut self, message: Message) -> Result<(), Error> {
        let (state, conn) = unsafe {&mut *self.conn.get()};
        state.assert_alive();

        if matches!(message, Message::Close(_)) {*state = State::Closed}
        send(message, conn, &self.config, &mut self.n_buffered).await
    }

    pub async fn write(&mut self, message: Message) -> Result<usize, Error> {
        let (state, conn) = unsafe {&mut *self.conn.get()};
        state.assert_alive();

        if matches!(message, Message::Close(_)) {*state = State::Closed}
        write(message, conn, &self.config, &mut self.n_buffered).await
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        let (state, conn) = unsafe {&mut *self.conn.get()};
        state.assert_alive();

        flush(conn, &mut self.n_buffered).await
    }
}


#[cfg(feature="rt_tokio")]
pub mod split {
    use super::*;
    use tokio::net::{TcpStream, tcp::{ReadHalf as TcpReadHalf, WriteHalf as TcpWriteHalf}};


    impl Connection<TcpStream> {
        pub fn split(self) -> (ReadHalf, WriteHalf) {
            let (state, conn) = unsafe {&mut *self.conn.get()};
            state.assert_alive();
    
            let (read, write) = conn.split();
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
