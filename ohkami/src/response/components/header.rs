pub(crate) struct ResponseHeaders(
    String
); impl ResponseHeaders {
    #[inline] pub(crate) fn new() -> Self {
        Self(String::new())
    }
    #[inline] pub(crate) fn append(&mut self, key: &'static str, value: &'static str) {
        self.0 += key;
        self.0 += ": ";
        self.0 += value;
        self.0 += "\r\n";
    }
    #[inline] pub(in super::super) fn as_str(&self) -> &str {
        &self.0
    }
}
