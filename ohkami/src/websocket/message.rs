use std::{borrow::Cow, io::{Error, ErrorKind}};
use crate::{__rt__::{AsyncReader, AsyncWriter}};
use super::{frame::{Frame, OpCode, CloseCode}, Config};


const PING_PONG_PAYLOAD_LIMIT: usize = 125;

pub enum Message {
    Text  (String),
    Binary(Vec<u8>),
    Ping  (Vec<u8>),
    Pong  (Vec<u8>),
    Close (Option<CloseFrame>),
}
pub struct CloseFrame {
    pub code:   CloseCode,
    pub reason: Option<Cow<'static, str>>,
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
    pub(crate) fn into_frame(self) -> Frame {
        let (opcode, payload) = match self {
            Message::Text  (text)  => (OpCode::Text,   text.into_bytes()),
            Message::Binary(bytes) => (OpCode::Binary, bytes),
            Message::Ping(mut bytes) => {
                bytes.truncate(PING_PONG_PAYLOAD_LIMIT);
                (OpCode::Ping, bytes)
            }
            Message::Pong(mut bytes) => {
                bytes.truncate(PING_PONG_PAYLOAD_LIMIT);
                (OpCode::Ping, bytes)
            }
            Message::Close(close_frame) => {
                let payload = close_frame
                    .map(|CloseFrame { code, reason }| {
                        let code   = code.into_bytes();
                        let reason = reason.as_ref().map(|cow| cow.as_bytes()).unwrap_or(&[]);
                        [&code, reason].concat()
                    }).unwrap_or(Vec::new());
                (OpCode::Close, payload)
            }
        };

        Frame { is_final: false, opcode, payload }
    }

    pub(crate) async fn write(self,
        stream: &mut (impl AsyncWriter + Unpin),
        config: &Config,
    ) -> Result<usize, Error> {
        self.into_frame().write_unmasked(stream, config).await
    }
//    /// for test
//    pub(crate) async fn masking_write(self,
//        stream: &mut (impl AsyncWriter + Unpin),
//        config: &Config,
//        mask:   [u8; 4],
//    ) -> Result<usize, Error> {
//        let mut frame = self.into_frame();
//        frame.mask = Some(mask);
//        frame.write_masked(stream, config).await
//    }
}

impl Message {
    pub(crate) async fn read_from(
        stream: &mut (impl AsyncReader + Unpin),
        config: &Config,
    ) -> Result<Option<Self>, Error> {
        let first_frame = match Frame::read_from(stream, config).await? {
            Some(frame) => frame,
            None        => return Ok(None),
        };

        match &first_frame.opcode {
            OpCode::Text => {
                let mut payload = String::from_utf8(first_frame.payload)
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Text frame's payload is not valid UTF-8"))?;
                if !first_frame.is_final {
                    while let Ok(Some(next_frame)) = Frame::read_from(stream, config).await {
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

                if let Some(limit) = &config.max_message_size {
                    (&payload.len() <= limit).then_some(())
                        .ok_or_else(|| Error::new(
                            ErrorKind::InvalidData,
                            "Incoming message is too large"
                        ))?;
                }

                Ok(Some(Message::Text(payload)))
            }
            OpCode::Binary => {
                let mut payload = first_frame.payload;
                if !first_frame.is_final {
                    while let Ok(Some(mut next_frame)) = Frame::read_from(stream, config).await {
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

                if let Some(limit) = &config.max_message_size {
                    (&payload.len() <= limit).then_some(())
                        .ok_or_else(|| Error::new(
                            ErrorKind::InvalidData,
                            "Incoming message is too large"
                        ))?;
                }

                Ok(Some(Message::Binary(payload)))
            }

            OpCode::Ping => {
                let payload = first_frame.payload;
                (payload.len() <= PING_PONG_PAYLOAD_LIMIT)
                    .then_some(Some(Message::Ping(payload)))
                    .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Incoming ping payload is too large"))
            }
            OpCode::Pong => {
                let payload = first_frame.payload;
                (payload.len() <= PING_PONG_PAYLOAD_LIMIT)
                    .then_some(Some(Message::Pong(payload)))
                    .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Incoming pong payload is too large"))
            }

            OpCode::Close => {
                let payload = first_frame.payload;
                Ok(Some(Message::Close(
                    (! payload.is_empty()).then(|| {
                        let (code_bytes, rem) = payload.split_at(2);
                        let code   = CloseCode::from_bytes(unsafe {(code_bytes.as_ptr() as *const [u8; 2]).read()});
                        let reason = (! rem.is_empty()).then(|| Cow::Owned(String::from_utf8(rem.to_vec()).unwrap()));
                        CloseFrame { code, reason }
                    })
                )))
            }

            OpCode::Continue => Err(Error::new(ErrorKind::InvalidData, "Unexpected continue frame"))
        }
    }
}
