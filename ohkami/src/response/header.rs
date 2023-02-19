use std::fmt::Display;

pub(crate) struct ResponseHeaders(
    pub(crate) String
);
impl ResponseHeaders {
    #[inline] pub(crate) fn set(&mut self, key: &'static str, value: &'static str) {
        self.0 += key;
        self.0 += ": ";
        self.0 += value;
        self.0 += "\r\n";
    }
}
