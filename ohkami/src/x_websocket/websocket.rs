use std::io::Error;
use super::Message;
use crate::__rt__::{TcpStream};


pub struct WebSocket {
    stream: TcpStream
}

impl WebSocket {
    pub(crate) fn new(stream: TcpStream) -> Self {
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

#[cfg(feature="rt_tokio")] const _: () = {
    impl WebSocket {
        pub fn split(&mut self) -> (ReadHalf, WriteHalf) {
            let (rh, wh) = self.stream.split();
            (ReadHalf(rh), WriteHalf(wh))
        }
    }

    
    use crate::__rt__::{
        ReadHalf as TcpReadHalf,
        WriteHalf as TcpWriteHalf,
    };

    pub struct ReadHalf<'ws>(TcpReadHalf<'ws>);
    impl<'ws> ReadHalf<'ws> {
        pub async fn recv(&mut self) -> Result<Option<Message>, Error> {
            Message::read_from(&mut self.0).await
        }
    }

    pub struct WriteHalf<'ws>(TcpWriteHalf<'ws>);
    impl<'ws> WriteHalf<'ws> {
        pub async fn send(&mut self, message: Message) -> Result<(), Error> {
            message.send(&mut self.0).await
        }
    }
};
