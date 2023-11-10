#[cfg(not(target_pointer_width = "64"))]
compile_error!{ "pointer width must be 64" }

mod context;
mod message;
mod upgrade;
mod frame;
mod sign;

use std::{io::{Error, ErrorKind}, sync::{Arc, OnceLock, atomic::AtomicUsize}, collections::HashMap};
use crate::__rt__::{TcpStream, RwLock};
use self::message::Message;

pub(crate) use upgrade::{UpgradeID, request_upgrade_id, reserve_upgrade, assume_upgraded};


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
