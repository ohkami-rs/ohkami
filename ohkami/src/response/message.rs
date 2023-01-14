use std::borrow::Cow;

use super::body::Body;

pub trait Message {
    fn as_message(self) -> Cow<'static, str>;
}
impl Message for String {
    fn as_message(self) -> Cow<'static, str> {
        Cow::Owned(self)
    }
}
impl Message for &String {
    fn as_message(self) -> Cow<'static, str> {
        Cow::Owned(self.to_owned())
    }
}
impl Message for &'static str {
    fn as_message(self) -> Cow<'static, str> {
        Cow::Borrowed(self)
    }
}

pub trait ErrorMessage {
    fn as_error_message(self) -> Option<Body>;
}
impl<Msg: Message> ErrorMessage for Msg {
    fn as_error_message(self) -> Option<Body> {
        Some(Body::text_plain(self.as_message()))
    }
}
impl ErrorMessage for Option<String> {
    fn as_error_message(self) -> Option<Body> {
        self.map(|msg| Body::text_plain(msg.as_message()))
    }
}