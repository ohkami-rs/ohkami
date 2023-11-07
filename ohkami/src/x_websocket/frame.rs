use std::io::{Error, ErrorKind};
use crate::__rt__::{AsyncReader};


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
} impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {match byte {
        0x0 => Self::Continue, 0x1 => Self::Text, 0x2 => Self::Binary,
        0x8 => Self::Close,    0x9 => Self::Ping, 0xa => Self::Pong,
        // 0x3..=0x7 | 0xb..=0xf => Self::Reserved,
        _ => panic!("OpCode out of range: {byte}")
    }}
}

pub enum CloseCode {
    Normal, Away, Protocol, Unsupported, Status, Abnormal, Invalid,
    Policy, Size, Extension, Error, Restart,Again, Tls, Reserved,
    Iana(u16), Library(u16), Bad(u16),
} impl From<u16> for CloseCode {
    fn from(code: u16) -> Self {match code {
        1000 => Self::Normal, 1001 => Self::Away,      1002 => Self::Protocol, 1003 => Self::Unsupported,
        1005 => Self::Status, 1006 => Self::Abnormal,  1007 => Self::Invalid,  1008 => Self::Policy,
        1009 => Self::Size,   1010 => Self::Extension, 1011 => Self::Error,    1012 => Self::Restart,
        1013 => Self::Again,  1015 => Self::Tls,       1016..=2999 => Self::Reserved,
        3000..=3999 => Self::Iana(code),   4000..=4999 => Self::Library(code),    _ => Self::Bad(code),
    }}
}

pub struct Frame {
    pub is_final: bool,
    pub opcode:   OpCode,
    pub mask:     Option<[u8; 4]>,
    pub payload:  Vec<u8>,
} impl Frame {
    pub async fn read_from(stream: &mut (impl AsyncReader + Unpin)) -> Result<Option<Self>, Error> {
        let [first, second] = {
            let mut head = [0; 2];
            stream.read_exact(&mut head).await?;
            head
        };

        let is_final = first & 0x80 != 0;
        let opcode   = OpCode::from(first & 0x0F);

        let payload_len = {
            let payload_len_byte = second & 0x7F;
            let len_part_size = match payload_len_byte {126=>2, 127=>8, _=>0};
            match len_part_size {
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
            }
        };

        let mask = if second & 0x80 == 0 {None} else {
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
}
