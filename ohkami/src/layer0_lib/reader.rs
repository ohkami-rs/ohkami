pub(crate) struct Reader<'b> {
    bytes: &'b [u8],
    pos:   usize,
} impl<'b> Reader<'b> {
    pub(crate) fn new(bytes: &'b [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes[self.pos..]
    }
} impl<'b> Reader<'b> {
    pub(crate) fn read_until(&mut self, byte: u8) -> Option<&'b [u8]> {
        let current_pos = self.pos;
        for b in &self.bytes[current_pos..] {
            self.pos += 1;
            if &byte == b {return Some(&self.bytes[current_pos..(self.pos)])}
        }
        None
    }
    pub(crate) fn read_before(&mut self, byte: u8) -> Option<&'b [u8]> {
        let current_pos = self.pos;
        /* `byte` は読み飛ばす */
        for b in &self.bytes[current_pos..] {
            self.pos += 1;
            if &byte == b {return Some(&self.bytes[current_pos..(self.pos-1)])}
        }
        None
    }
    #[inline] pub(crate) fn read_prefix(&mut self, prefix: &[u8]) -> Option<()> {
        {prefix == &self.bytes[(self.pos)..(self.pos + prefix.len())]}
            .then(|| self.pos += prefix.len())
    }
    pub(crate) fn read_prefix_oneof<const N: usize>(&mut self, prefixes: [&[u8]; N]) -> Option<()> {
        for prefix in prefixes {
            if self.read_prefix(prefix).is_some() {return Some(())}
        }
        None
    }
}




#[cfg(test)] mod test {
    use super::Reader;

    #[test] fn test_read_until() {
        let mut r = Reader::new(b"Hello, world!");
        let read = r.read_until(b' ');
        assert_eq!(read,      Some(&b"Hello, "[..]));
        assert_eq!(r.bytes(), b"world!");

        let mut r = Reader::new(b"Hello, world!");
        let read = r.read_until(b'o');
        assert_eq!(read,      Some(&b"Hello"[..]));
        assert_eq!(r.bytes(), b", world!");
    }

    #[test] fn test_read_before() {
        let mut r = Reader::new(b"Hello, world!");
        let read = r.read_before(b' ');
        assert_eq!(read,      Some(&b"Hello,"[..]));
        assert_eq!(r.bytes(), b"world!");

        let mut r = Reader::new(b"Hello, world!");
        let read = r.read_before(b'o');
        assert_eq!(read,      Some(&b"Hell"[..]));
        assert_eq!(r.bytes(), b", world!");
    }

    #[test] fn test_read_prefix() {
        let mut r = Reader::new(b"Hello, world!");
        assert!(r.read_prefix(b"Hello").is_some());
        assert_eq!(r.bytes(), b", world!");

        let mut r = Reader::new(b"Hello, world!");
        assert!(r.read_prefix(b"Hello ").is_none());
        assert_eq!(r.bytes(), b"Hello, world!");
    }
}

