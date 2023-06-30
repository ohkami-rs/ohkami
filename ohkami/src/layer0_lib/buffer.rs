use std::{ops::Index};
use crate::{__dep__::{TcpStream, AsyncReader}};

pub(crate) const BUFFER_SIZE: usize = 1024;
pub(crate) type BufRange = std::ops::Range<usize>;

pub(crate) struct Buffer(
    [u8; BUFFER_SIZE]
);

impl Buffer {
    #[cfg(test)]
    pub(crate) fn from_raw_str(req_str: &str) -> Self {
        let mut raw_buffer = [b'\0'; BUFFER_SIZE];
        for (i, b) in req_str.as_bytes().into_iter().enumerate() {
            raw_buffer[i] = *b
        }
        Self(raw_buffer)
    }

    pub(crate) async fn new(stream: &mut TcpStream) -> Self {
        let mut raw_buffer = [b'\0'; BUFFER_SIZE];
        if let Err(e) = stream.read(&mut raw_buffer).await {
            panic!("Failed to read stream: {e}")
        }
        Self(raw_buffer)
    }

    pub(crate) fn read_str(&self, range: &BufRange) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(
                &self.0[(range.start)..(range.end)]
            )
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
