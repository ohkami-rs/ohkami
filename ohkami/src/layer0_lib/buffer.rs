use std::{ops::Index};
use crate::{__dep__::{TcpStream, AsyncReader}};

pub(crate) const METADATA_SIZE: usize = 1024;
pub(crate) const PAYLOAD_LIMIT: usize = 65536;
pub(crate) type BufRange = std::ops::Range<usize>;

pub(crate) struct Buffer {
    metadata: [u8; METADATA_SIZE],
    payload:  Vec<u8>,
}

#[allow(unused)]
impl Buffer {
    pub(crate) fn new() -> Self {
        Self {
            metadata: [b'0'; METADATA_SIZE],
            payload:  Vec::with_capacity(0),
        }
    }
}

const _: () = {
    impl Index<BufRange> for Buffer {
        type Output = [u8];
        #[inline(always)] fn index(&self, range: BufRange) -> &Self::Output {
            &self.0[range]
        }
    }
    impl<'range> Index<&'range BufRange> for Buffer {
        type Output = [u8];
        #[inline(always)] fn index(&self, range: &'range BufRange) -> &Self::Output {
            &self.0[(range.start)..(range.end)]
        }
    }

    impl Index<std::ops::RangeFrom<usize>> for Buffer {
        type Output = [u8];
        #[inline(always)] fn index(&self, range: std::ops::RangeFrom<usize>) -> &Self::Output {
            &self.0[range]
        }
    }

    impl Index<usize> for Buffer {
        type Output = u8;
        #[inline(always)] fn index(&self, index: usize) -> &Self::Output {
            &self.0[index]
        }
    }
};

#[cfg(test)]
const _: () = {
    impl Clone for Buffer {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl std::fmt::Debug for Buffer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{}]", {
                let mut elems = String::new();
                for b in &self.0 {
                    match b {
                        b'\0' => {
                            elems.pop(/* final ' ' */);
                            elems.pop(/* final ',' */);
                        },
                        _ => {
                            elems.push_str(&b.to_string());
                            elems.push(',');
                            elems.push(' ');
                        },
                    }
                }
                elems
            })
        }
    }

    impl PartialEq for Buffer {
        fn eq(&self, other: &Self) -> bool {
            &self.0 == &other.0
        }
    }
};
