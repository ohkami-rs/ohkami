pub(crate) struct Reader<'b> {
    bytes: &'b [u8],
    pos:   usize,
} impl<'b> Reader<'b> {
    pub(crate) fn new(bytes: &'b [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    #[inline] pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes[self.pos..]
    }
} impl<'b> Reader<'b> {
    #[inline] pub(crate) fn read(&mut self, byte: u8) -> Option<()> {
        (&byte == self.bytes().first()?)
            .then(|| self.pos += 1)
    }
    #[inline] pub(crate) fn read_(&mut self, bytes: &[u8]) -> Option<()> {
        (bytes == &self.bytes()[..bytes.len()])
            .then(|| self.pos += bytes.len())
    }
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
        for b in &self.bytes[current_pos..] {
            if &byte == b {return Some(&self.bytes[current_pos..(self.pos)])}
            self.pos += 1;
        }
        None
    }
    pub(crate) fn read_before_(&mut self, bytes: &[u8]) -> Option<&'b [u8]> {
        let current_pos = self.pos;
        if current_pos + bytes.len() > self.bytes.len() {return None}

        for i in current_pos..(self.bytes.len() - bytes.len()) {
            if &self.bytes[i..(i+bytes.len())] == bytes {
                return Some(&self.bytes[current_pos..i])
            }
            self.pos += 1;
        }

        None
    }
    pub(crate) fn read_split_left(&mut self, byte: u8) -> Option<&'b [u8]> {
        let current_pos = self.pos;
        for b in &self.bytes[current_pos..] {
            self.pos += 1;
            if &byte == b {return Some(&self.bytes[current_pos..(self.pos-1)])}
        }
        None
    }
    pub(crate) fn read_prefix_oneof<const N: usize>(&mut self, prefixes: [&[u8]; N]) -> Option<usize> {
        for i in 0..(prefixes.len()) {
            if self.read_(&prefixes[i]).is_some() {return Some(i)}
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

    #[test] fn test_read_split_left() {
        let mut r = Reader::new(b"Hello, world!");
        let read = r.read_split_left(b' ');
        assert_eq!(read,      Some(&b"Hello,"[..]));
        assert_eq!(r.bytes(), b"world!");

        let mut r = Reader::new(b"Hello, world!");
        let read = r.read_split_left(b'o');
        assert_eq!(read,      Some(&b"Hell"[..]));
        assert_eq!(r.bytes(), b", world!");
    }

    #[test] fn test_read_before() {
        let mut r = Reader::new(b"Hello, world!");
        let read = r.read_before(b' ');
        assert_eq!(read,      Some(&b"Hello,"[..]));
        assert_eq!(r.bytes(), b" world!");

        let mut r = Reader::new(b"Hello, world!");
        let read = r.read_before(b'o');
        assert_eq!(read,      Some(&b"Hell"[..]));
        assert_eq!(r.bytes(), b"o, world!");
    }

    #[test] fn test_read_before_() {
        todo!()
    }

    #[test] fn test_read_() {
        let mut r = Reader::new(b"Hello, world!");
        assert!(r.read_(b"Hello").is_some());
        assert_eq!(r.bytes(), b", world!");

        let mut r = Reader::new(b"Hello, world!");
        assert!(r.read_(b"Hello ").is_none());
        assert_eq!(r.bytes(), b"Hello, world!");
    }
}

