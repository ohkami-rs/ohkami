pub(crate) struct ResponseHeaders(String);
impl ResponseHeaders {
    #[inline] pub(crate) fn add(&mut self, key: &'static str, value: &'static str) {
        self.0 += key;
        self.0 += ": ";
        self.0 += value;
        self.0 += "\r\n";
    }
}
