use super::name::*;
use super::value::*;


pub struct ClientHeaders {
    values: [Option<HeaderValue>; N_CLIENT_HEADERS],
}

impl ClientHeaders {
    #[inline] pub(crate) fn insert(&mut self, name: ClientHeader, value: HeaderValue) {
        self.values[name as usize] = Some(value)
    }

    pub(crate) fn append(&mut self, name: ClientHeader, value: HeaderValue) {
        let index = name as usize;
        match &mut self.values[index] {
            None    => self.values[index] = Some(value),
            Some(v) => v.append(value),
        }
    }

    pub(crate) fn remove(&mut self, name: ClientHeader) {
        self.values[name as usize] = None;
    }

    #[inline] pub(crate) fn get(&self, name: ClientHeader) -> Option<&str> {
        match &self.values[name as usize] {
            Some(v) => Some(v.as_str()),
            None => None,
        }
    }
}
impl ClientHeaders {
    pub(crate) fn init() -> Self {
        Self { values: std::array::from_fn(|_| None) }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        struct Iter<'i> {
            map: &'i ClientHeaders,
            cur: usize,
        }
        impl<'i> Iterator for Iter<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                for i in self.cur..N_CLIENT_HEADERS {
                    if let Some(v) = &self.map.values[i] {
                        self.cur = i + 1;
                        return Some((&CLIENT_HEADERS[i].as_str(), v.as_str()))
                    }
                }
                None
            }
        }

        Iter { map: self, cur: 0 }
    }
}


pub struct ServerHeaders {
    values: [Option<HeaderValue>; N_SERVER_HEADERS],

    /// Size of whole the byte stream when this is written into HTTP response.
    size: usize,

    // 
}

impl ServerHeaders {
    #[inline] pub(crate) fn insert(&mut self, name: ServerHeader, value: HeaderValue) {
        let (name_len, value_len) = (name.as_bytes().len(), value.as_str().len());
        match self.values[name as usize].replace(value) {
            None       => self.size += name_len + ": ".len() + value_len + "\r\n".len(),
            Some(prev) => {
                let prev_len = prev.as_str().len();
                if value_len > prev_len {
                    self.size += value_len - prev_len;
                } else {
                    self.size -= prev_len - value_len;
                }
            }
        }
    }

    pub(crate) fn append(&mut self, name: ServerHeader, value: HeaderValue) {
        let name_len = name.as_bytes().len();
        let index = name as usize;
        match &mut self.values[index] {
            None => {
                self.size += name_len + ": ".len() + value.as_str().len() + "\r\n".len();
                self.values[index] = Some(value);
            }
            Some(v) => {
                let before = v.as_str().len();
                v.append(value);
                self.size += v.as_str().len() - before;
            }
        }
    }

    #[inline] pub(crate) fn remove(&mut self, name: ServerHeader) {
        let name_len = name.as_bytes().len();
        let v = &mut self.values[name as usize];
        if let Some(v) = v {
            self.size -= name_len + ": ".len() + v.as_str().len() + "\r\n".len()
        }
        *v = None;
    }

    pub(crate) fn get(&self, name: ServerHeader) -> Option<&str> {
        self.values[name as usize].as_ref().map(HeaderValue::as_str)
    }
}
impl ServerHeaders {
    pub(crate) fn new() -> Self {
        Self {
            values: std::array::from_fn(|_| None),
            size:   "\r\n".len(),
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        struct Iter<'i> {
            map: &'i ServerHeaders,
            cur: usize,
        }
        impl<'i> Iterator for Iter<'i> {
            type Item = (&'i str, &'i str);
            fn next(&mut self) -> Option<Self::Item> {
                for i in self.cur..N_SERVER_HEADERS {
                    if let Some(v) = &self.map.values[i] {
                        self.cur = i + 1;
                        return Some((SERVER_HEADERS[i].as_str(), v.as_str()))
                    }
                }
                None
            }
        }

        Iter { map: self, cur: 0 }
    }

    pub(crate) fn write_to(self, buf: &mut Vec<u8>) {
        macro_rules! write {
            ($buf:ident <- $bytes:expr) => {
                unsafe {
                    let (buf_len, bytes_len) = ($buf.len(), $bytes.len());
                    std::ptr::copy_nonoverlapping(
                        $bytes.as_ptr(),
                        $buf.as_mut_ptr().add(buf_len),
                        bytes_len
                    );
                    $buf.set_len(buf_len + bytes_len);
                }
            };
        }

        buf.reserve(self.size);

        for h in &SERVER_HEADERS {
            if let Some(v) = &self.values[*h as usize] {
                write!(buf <- h.as_bytes());
                write!(buf <- b": ");
                write!(buf <- v.as_str().as_bytes());
                write!(buf <- b"\r\n");
            }
        }
        write!(buf <- b"\r\n");
    }
}
