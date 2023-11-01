use std::borrow::Cow;


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
