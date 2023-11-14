use std::io::{Error, ErrorKind};
use crate::__rt__::{AsyncReader, AsyncWriter};
use super::websocket::Config;


#[derive(PartialEq)]
pub enum OpCode {
    /* data op codes */
    Continue /* 0x0 */,
    Text     /* 0x1 */,
    Binary   /* 0x2 */,
    /* control op codes */
    Close    /* 0x8 */,
    Ping     /* 0x9 */,
    Pong     /* 0xa */,
    /* reserved op codes */
    // Reserved /* 0x[3-7,b-f] */,
} impl OpCode {
    fn from_byte(byte: u8) -> Result<Self, Error> {
        Ok(match byte {
            0x0 => Self::Continue, 0x1 => Self::Text, 0x2 => Self::Binary,
            0x8 => Self::Close,    0x9 => Self::Ping, 0xa => Self::Pong,
            0x3..=0x7 | 0xb..=0xf => return Err(Error::new(
                ErrorKind::Unsupported, "Ohkami doesn't handle reserved opcodes")),
            _ => return Err(Error::new(
                ErrorKind::InvalidData, "OpCode out of range")),
        })
    }
    fn into_byte(self) -> u8 {
        match self {
            Self::Continue => 0x0, Self::Text => 0x1, Self::Binary => 0x2,
            Self::Close    => 0x8, Self::Ping => 0x9, Self::Pong   => 0xa,
        }
    }
}

pub enum CloseCode {
    Normal, Away, Protocol, Unsupported, Status, Abnormal, Invalid,
    Policy, Size, Extension, Error, Restart, Again, Tls, Reserved,
    Iana(u16), Library(u16), Bad(u16),
} impl CloseCode {
    pub(super) fn from_bytes(bytes: [u8; 2]) -> Self {
        let code = u16::from_be_bytes(bytes);
        match code {
            1000 => Self::Normal, 1001 => Self::Away,      1002 => Self::Protocol, 1003 => Self::Unsupported,
            1005 => Self::Status, 1006 => Self::Abnormal,  1007 => Self::Invalid,  1008 => Self::Policy,
            1009 => Self::Size,   1010 => Self::Extension, 1011 => Self::Error,    1012 => Self::Restart,
            1013 => Self::Again,  1015 => Self::Tls,       1016..=2999 => Self::Reserved,
            3000..=3999 => Self::Iana(code),   4000..=4999 => Self::Library(code),    _ => Self::Bad(code),
        }
    }
    pub(super) fn into_bytes(self) -> [u8; 2] {
        match self {
            
        }
    }
}

pub struct Frame {
    pub is_final: bool,
    pub opcode:   OpCode,
    pub mask:     Option<[u8; 4]>,
    pub payload:  Vec<u8>,
} impl Frame {
    pub(super) async fn read_from(
        stream: &mut (impl AsyncReader + Unpin),
        config: &Config,
    ) -> Result<Option<Self>, Error> {
        let [first, second] = {
            let mut head = [0; 2];
            stream.read_exact(&mut head).await?;
            head
        };

        let is_final = first & 0x80 != 0;
        let opcode   = OpCode::from_byte(first & 0x0F)?;

        let payload_len = {
            let payload_len_byte = second & 0x7F;
            let len_part_size = match payload_len_byte {127=>8, 126=>2, _=>0};

            let len = match len_part_size {
                0 => payload_len_byte as usize,
                _ => {
                    let mut bytes = [0; 8];
                    if let Err(e) = stream.read_exact(&mut bytes[(8 - len_part_size)..]).await {
                        return match e.kind() {
                            ErrorKind::UnexpectedEof => Ok(None),
                            _                        => Err(e.into()),
                        }
                    }
                    usize::from_be_bytes(bytes)
                }
            }; if let Some(limit) = &config.max_frame_size {
                (&len <= limit).then_some(())
                    .ok_or_else(|| Error::new(
                        ErrorKind::InvalidData,
                        "Incoming frame is too large"
                    ))?;
            }

            len
        };

        let mask = if second & 0x80 == 0 {
            (config.accept_unmasked_frames).then_some(None)
                .ok_or_else(|| Error::new(
                    ErrorKind::InvalidData,
                    "Client frame is unmasked"
                ))?
        } else {
            let mut mask_bytes = [0; 4];
            if let Err(e) = stream.read_exact(&mut mask_bytes).await {
                return match e.kind() {
                    ErrorKind::UnexpectedEof => Ok(None),
                    _                        => Err(e.into()),
                }
            }
            Some(mask_bytes)
        };

        let payload = {
            let mut payload = Vec::with_capacity(payload_len);
            stream.read_exact(&mut payload).await?;
            payload
        };

        Ok(Some(Self { is_final, opcode, mask, payload }))
    }

    pub(super) async fn write_to(self,
        stream:  &mut (impl AsyncWriter + Unpin),
        _config: &Config,
    ) -> Result<usize, Error> {
        fn into_bytes(frame: Frame) -> Vec<u8> {
            let Frame { is_final, opcode, mask, payload } = frame;

            let (payload_len_byte, payload_len_bytes) = match payload.len() {
                ..=125      => (payload.len() as u8, None),
                126..=65535 => (126, Some((|| (payload.len() as u16).to_be_bytes().to_vec())())),
                _           => (127, Some((|| (payload.len() as u64).to_be_bytes().to_vec())())),
            };

            let first  = is_final.then_some(1).unwrap_or(0)       << 7 + opcode.into_byte();
            let second = mask.is_some().then_some(1).unwrap_or(0) << 7 + payload_len_byte;

            let mut header_bytes = vec![first, second];
            if let Some(mut payload_len_bytes) = payload_len_bytes {
                header_bytes.append(&mut payload_len_bytes)
            }
            if let Some(mask_bytes) = mask {
                header_bytes.extend(mask_bytes)
            }

            [header_bytes, payload].concat()
        }

        stream.write(&into_bytes(self)).await
    }
}
