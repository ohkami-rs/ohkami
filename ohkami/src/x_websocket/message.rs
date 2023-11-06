use std::{borrow::Cow, io::Result};
use crate::{__rt__::AsyncReader};


pub enum Message {
    Text  (String),
    Binary(Vec<u8>),
    Ping  (PingPongFrame),
    Pong  (PingPongFrame),
    Close (Option<CloseFrame>),
}
pub struct PingPongFrame {
    buf: [u8; 125],
    len: usize/* less than 125 */
}
pub struct CloseFrame {
    pub code:   u16,
    pub reason: Cow<'static, str>,
}

const _: (/* `From` impls */) = {
    impl From<&str> for Message {
        fn from(string: &str) -> Self {
            Self::Text(string.to_string())
        }
    }
    impl From<String> for Message {
        fn from(string: String) -> Self {
            Self::Text(string)
        }
    }
    impl From<&[u8]> for Message {
        fn from(data: &[u8]) -> Self {
            Self::Binary(data.to_vec())
        }
    }
    impl From<Vec<u8>> for Message {
        fn from(data: Vec<u8>) -> Self {
            Self::Binary(data)
        }
    }
};

impl Message {
    pub(super) async fn from(stream: impl AsyncReader + Unpin) -> Result<Self> {
        
    }
}
