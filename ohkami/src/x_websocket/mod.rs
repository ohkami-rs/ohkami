#[cfg(not(target_pointer_width = "64"))]
compile_error!{ "pointer width must be 64" }

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
    async fn handle(self, handle_message: impl Fn(Message) -> Message) {
        let stream = &mut *self.stream.lock().await;
        //while let Some(Ok(_)) = stream.re;
    }
}
