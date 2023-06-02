#![allow(non_snake_case)]

use crate::layer0_lib::{now, AsStr, ContentType};


pub(crate) struct ResponseHeaders {
    ContentType: Option<ContentType>,
    others: String,
}

impl ResponseHeaders {
    #[inline(always)] pub(crate) fn new() -> Self {
        Self {
            ContentType: None,
            others: String::from(
"Connection: Keep-Alive\r
Keep-Alive: timeout=5")
        }
    }
    #[inline(always)] pub(crate) fn as_str(&mut self) -> &str {
        let Self { ContentType:content_type, others:h } = self;
        if let Some(content_type) = content_type {
            h.push_str("\r\nContent-Type: ");
            h.push_str(content_type.as_str());
        }
        h
    }
    #[inline(always)] pub(crate) fn others_than_ContentType(&self) -> &str {
        &self.others
    }
}

impl ResponseHeaders {
    #[inline] pub(crate) fn append(&mut self, key: &str, value: impl AsStr) {
        self.others.push('\r');
        self.others.push('\n');
        self.others.push_str(key);
        self.others.push(':');
        self.others.push(' ');
        self.others.push_str(value.as_str());
    }

    pub(crate) fn clear(&mut self, key: &str) {
        let mut pos = 0;
        for (k, v) in self.others.lines().map(|kv| unsafe {kv.split_once(':').unwrap_unchecked()}) {
            if k != key {
                pos += k.len() + 1/* ':' */ + v/* with ' ' */.len() + 2/* "\r\n" */
            } else {
                let prev_end = pos - 2/* "\r\n" */;
                let mut to = prev_end + k.len() + 1/*':'*/ + v/*with ' '*/.len() - 1;
                if v.ends_with('\n') {to += 2/* "\r\n" */};
                self.others.replace_range(prev_end..to, "");
                return
            }
        }
    }
    pub(crate) fn clearContentType(&mut self) {
        self.ContentType = None;
    }

    pub(crate) fn set(&mut self, key: &str, value: impl AsStr) {
        let mut pos = 0;
        let mut v_range = None;

        for (k, v) in self.others.lines().map(|kv| unsafe {kv.split_once(':').unwrap_unchecked()}) {
            if k != key {
                pos += k.len() + 1/* ':' */ + v/* with ' ' */.len() + 2/* "\r\n" */
            } else {
                let value_start = pos + k.len() + 2/* ": " */;
                v_range.replace(value_start..(value_start+v.len()-1));
                break
            }
        }

        match v_range {
            Some(range) => self.others.replace_range(range, value.as_str()),
            None => self.append(key, value)
        }
    }
    pub(crate) fn setContentType(&mut self, content_type: ContentType) {
        self.ContentType.replace(content_type);
    }

    #[inline] pub(crate) fn append_if_not_has(&mut self, key: &str, value: impl AsStr) {
        for (k, v) in self.others.lines().map(|kv| unsafe {kv.split_once(':').unwrap_unchecked()}) {
            if k == key {
                return
            }
        }
        self.append(key, value)
    }
}




#[cfg(test)] #[test] fn check_set_header() {
    let mut h = ResponseHeaders::new();
    h.set("Keep-Alive", "timeout=10");
    assert_eq!(h.as_str(),
"Connection: Keep-Alive\r
Keep-Alive: timeout=10"
    );

    let mut h = ResponseHeaders::new();
    h.append("Content-Language", "en-US");
    h.append("Content-Type", "application/json");
    h.set("Content-Type", "text/html");
    assert_eq!(h.as_str(),
"Connection: Keep-Alive\r
Keep-Alive: timeout=5\r
Content-Language: en-US\r
Content-Type: text/html"
    );
}


#[cfg(test)] #[test] fn check_clear_header() {
    let mut h = ResponseHeaders::new();
    h.clear("Connection");
    assert_eq!(h.as_str(),
"Keep-Alive: timeout=10"
    );

    let mut h = ResponseHeaders::new();
    h.append("Content-Language", "en-US");
    h.append("Content-Type", "application/json");

    h.clear("Content-Language");
    assert_eq!(h.as_str(),
"Connection: Keep-Alive\r
Keep-Alive: timeout=5\r
Content-Type: text/html"
    );

    h.clear("Content-Type");
    assert_eq!(h.as_str(),
"Connection: Keep-Alive\r
Keep-Alive: timeout=5"
    );
}
