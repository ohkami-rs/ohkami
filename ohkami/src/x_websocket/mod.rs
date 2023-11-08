#[cfg(not(target_pointer_width = "64"))]
compile_error!{ "pointer width must be 64" }

mod context;
mod message;
mod frame;
mod sign;

use std::io::{Error, ErrorKind};
use crate::__rt__::TcpStream;
use self::message::Message;


pub struct WebSocket {
    stream: TcpStream
}

impl WebSocket {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl WebSocket {
    pub async fn recv(&mut self) -> Result<Option<Message>, Error> {
        Message::read_from(&mut self.stream).await
    }
    pub async fn send(&mut self, message: Message) -> Result<(), Error> {
        message.send(&mut self.stream).await
    }
}
