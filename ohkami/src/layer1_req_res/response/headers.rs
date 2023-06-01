use crate::layer0_lib::{now, AsStr, ContentType};


pub(crate) struct ResponseHeaders(
    String,
);

impl ResponseHeaders {
    #[inline(always)] pub(crate) fn new() -> Self {
        Self(String::from(
"Connection: Keep-Alive\r
Keep-Alive: timeout=5"
        ))
    }
    #[inline] pub(crate) fn append(&mut self, key: &str, value: impl AsStr) {
        self.0.push('\r');
        self.0.push('\n');
        self.0.push_str(key);
        self.0.push(':');
        self.0.push(' ');
        self.0.push_str(value.as_str());
    }
    #[inline(always)] pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

// impl ResponseHeaders {
//     #[inline(always)] pub(crate) fn _date(mut self) -> Self {
//         self.append("Date", now());
//         self
//     }
//     #[inline(always)] pub(crate) fn _content_type(mut self, content_type: ContentType) -> Self {
//         self.append("Content-Type", content_type.as_str());
//         self
//     }
//     #[inline(always)] pub(crate) fn _content_length(mut self, content_length: usize) -> Self {
//         self.append("Content", content_length.to_string());
//         self
//     }
// }
// 