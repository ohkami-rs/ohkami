use std::{str::{self, Lines}, ops::{Index, RangeInclusive}};
use async_std::{net::TcpStream, io::ReadExt};
use crate::result::Result;

const BUF_SIZE: usize = 1024;


pub(crate) struct Buffer(
    [u8; BUF_SIZE]
); impl Buffer {
    pub async fn new(stream: &mut TcpStream) -> Result<Self> {
        let mut buffer = [b' '; BUF_SIZE];
        stream.read(&mut buffer).await?;
        Ok(Self(buffer))
    }
    pub fn lines(&self) -> Result<Lines> {
        Ok(str::from_utf8(&self.0)?.trim_end().lines())
    }
    pub fn read_str(&self, range: &BufRange) -> &str {
        let target_bytes = &self[*range];
        unsafe {
            std::str::from_utf8_unchecked(target_bytes)
        }
    } 

    pub(crate) async fn from_http_request_str(request: String) -> Self {
        let mut buffer = [b' '; BUF_SIZE];
        let mut request = request.as_bytes();
        assert!(request.len() <= BUF_SIZE, "ohkami can't handle request that's larger than {BUF_SIZE} bytes");
        request.read(&mut buffer).await.expect("failed to read request");
        Self(buffer)
    }
}
impl Index<BufRange> for Buffer {
    type Output = [u8];
    fn index(&self, range: BufRange) -> &Self::Output {
        &self.0[range.as_range()]
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BufRange(
    usize, usize
); impl BufRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self(start, end)
    }
    pub fn as_range(&self) -> RangeInclusive<usize> {
        self.0..=self.1
    }
}
