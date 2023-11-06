mod context;
mod message;
mod frame;
mod sign;

use std::{sync::Arc, io::Result};
use crate::__rt__::{TcpStream, Mutex, AsyncReader, AsyncWriter};
use self::message::Message;


pub struct WebSocket {
    stream: Arc<Mutex<TcpStream>>,
}

impl WebSocket {
    fn new(stream: Arc<Mutex<TcpStream>>) -> Self {
        Self { stream }
    }
}

impl WebSocket {
    //pub async fn recv(&self) -> Option<Result<Message>> {
    //    ( self.stream.lock().await)
    //    .rea
    //}
}
