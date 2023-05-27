use std::ops::Index;
use crate::{Error, __feature__::{TcpStream, StreamReader}};

pub(crate) const BUFFER_SIZE: usize = 1024;
pub(crate) type BufRange = std::ops::Range<usize>;


pub(crate) struct Buffer(
    [u8; BUFFER_SIZE]
);

impl Buffer {
    pub(crate) async fn new(stream: &mut TcpStream) -> Result<Self, Error> {
        let mut raw_buffer = [b'\0'; BUFFER_SIZE];
        stream.read(&mut raw_buffer).await?;
        Ok(Self(raw_buffer))
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
