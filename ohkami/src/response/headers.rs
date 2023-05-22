use std::borrow::Cow;


pub(crate) struct ResponseHeaders(
    String,
);

impl ResponseHeaders {
    pub(crate) fn new() -> Self {
        Self(String::from(
"Connection: Keep-Alive\r
Keep-Alive: timeout=5\r
"
        ))
    }
    pub(crate) fn add(&mut self, key: &'static str, value: Cow<'static, str>) {
        self.0.push_str(key);
        self.0.push(':');
        self.0.push(' ');
        self.0.push_str(&value);
        self.0.push('\r');
        self.0.push('\n');
    }
}

impl ResponseHeaders {
    pub(crate) fn _date(mut self) -> Self {
        
        self
    }
}
