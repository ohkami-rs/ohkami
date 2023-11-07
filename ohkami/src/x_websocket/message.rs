use std::{borrow::Cow, io::{Error, ErrorKind}};
use crate::{__rt__::AsyncReader};
use super::frame::{Frame, OpCode};


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
    pub(super) async fn read_from(stream: &mut (impl AsyncReader + Unpin)) -> Result<Option<Self>, Error> {
        let head_frame = match Frame::read_from(stream).await? {
            Some(frame) => frame,
            None        => return Ok(None),
        };

        match &head_frame.opcode {
            OpCode::Text => {
                let mut payload = String::from_utf8(head_frame.payload)
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Text frame's payload is not valid UTF-8"))?;
                if !head_frame.is_final {
                    while let Ok(Some(next_frame)) = Frame::read_from(stream).await {
                        if next_frame.opcode != OpCode::Continue {
                            return Err(Error::new(ErrorKind::InvalidData, "Expected continue frame"));
                        }
                        payload.push_str(std::str::from_utf8(&next_frame.payload)
                            .map_err(|_| Error::new(ErrorKind::InvalidData, "Text frame's payload is not valid UTF-8"))?
                        );
                        if next_frame.is_final {
                            break
                        }
                    }
                }
                Ok(Some(Message::Text(payload)))
            }
            OpCode::Binary => {
                let mut payload = head_frame.payload;
                if !head_frame.is_final {
                    while let Ok(Some(mut next_frame)) = Frame::read_from(stream).await {
                        if next_frame.opcode != OpCode::Continue {
                            return Err(Error::new(ErrorKind::InvalidData, "Expected continue frame"));
                        }
                        payload.append(
                            &mut next_frame.payload
                        );
                        if next_frame.is_final {
                            break
                        }
                    }
                }
                Ok(Some(Message::Binary(payload)))
            }
            OpCode::Ping => {
                todo!()
            }
            OpCode::Close    => return Ok(None),
            OpCode::Pong     => return Err(Error::new(ErrorKind::InvalidData, "Unexpected pong frame")),
            OpCode::Continue => return Err(Error::new(ErrorKind::InvalidData, "Unexpected continue frame")),
        }
    }
}
