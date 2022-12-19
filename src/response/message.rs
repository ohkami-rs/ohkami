use super::body::Body;

pub trait Message {
    fn as_message(self) -> String;
}
impl Message for String {
    fn as_message(self) -> String {
        self
    }
}
impl Message for &str {
    fn as_message(self) -> String {
        self.to_owned()
    }
}

pub trait ErrorMessage {
    fn as_message(self) -> Option<Body>;
}
impl<Msg: Message> ErrorMessage for Msg {
    fn as_message(self) -> Option<Body> {
        Some(Body::text_plain(self.as_message()))
    }
}
impl ErrorMessage for Option<String> {
    fn as_message(self) -> Option<Body> {
        self.map(|msg| Body::text_plain(msg))
    }
}