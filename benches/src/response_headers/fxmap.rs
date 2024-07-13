use std::{borrow::Cow, hash::BuildHasherDefault};

pub struct FxMap {
    map:  rustc_hash::FxHashMap<&'static str, Cow<'static, str>>,
    size: usize,
}
impl FxMap {
    pub fn new() -> Self {
        Self {
            map:  rustc_hash::FxHashMap::with_capacity_and_hasher(32, BuildHasherDefault::default()),
            size: 2/* "\r\n".len() */
        }
    }

    #[inline(always)]
    pub fn insert(
        &mut self,
        key:   &'static str,
        value: impl Into<Cow<'static, str>>,
    ) -> &mut Self {
        let value = value.into();

        self.size += value.len();
        if let Some(old) = self.map.insert(key, value) {
            self.size -= old.len();
        } else {
            self.size += key.len() + 2/* ": ".len() */ + 2/* "\r\n".len() */;
        }
        self
    }

    #[inline]
    pub fn remove(&mut self, key: &'static str) -> &mut Self {
        if let Some(old) = self.map.remove(&key) {
            self.size -= key.len() + 2/* ": ".len() */ + old.len() + 2/* "\r\n".len() */;
        }
        self
    }

    #[inline]
    pub fn write_to(&self, buf: &mut Vec<u8>) {
        macro_rules! push {
            ($buf:ident <- $bytes:expr) => {
                unsafe {
                    let (buf_len, bytes_len) = ($buf.len(), $bytes.len());
                    std::ptr::copy_nonoverlapping(
                        $bytes.as_ptr(),
                        <[u8]>::as_mut_ptr($buf).add(buf_len),
                        bytes_len
                    );
                    $buf.set_len(buf_len + bytes_len);
                }
            };
        }

        buf.reserve(self.size);

        for (k, v) in self.map.iter() {
            push!(buf <- k.as_bytes());
            push!(buf <- b": ");
            push!(buf <- v.as_bytes());
            push!(buf <- b"\r\n");
        }
        push!(buf <- b"\r\n")
    }
}
