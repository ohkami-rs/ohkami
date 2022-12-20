use crate::{result::{Result, ElseResponse}, response::Response};
use super::buffer::{Buffer, BufRange};


pub(crate) const RANGE_COLLECTION_SIZE: usize = 4;

#[derive(Debug)]
pub(crate) struct RangeMap(
    [Option<(BufRange, BufRange)>; RANGE_COLLECTION_SIZE]
); impl RangeMap {
    pub(crate) fn new() -> Self {
        Self([None, None, None, None])
    }
    pub(crate) fn insert(&mut self, index: usize, key: BufRange, value: BufRange) {
        self.0[index] = Some((key, value))
    }
    pub(crate) fn read_match_part_of_buffer<'map, 'key, 'buf>(
        &'map self,
        key:    &'key str,
        buffer: &'buf Buffer,
    ) -> Option<&'buf str> {
        let target_key = key.as_bytes();
        for key_value in &self.0 {
            if key_value.is_none() {return None}
            let (key, value) = key_value.as_ref().unwrap();
            if &buffer[*key] == target_key {
                return Some(buffer.read_str(value))
            }
        }
        None
    }

    pub(crate) fn debug_fmt_with(&self, buffer: &Buffer) -> String {
        let mut fmt = String::from("[");
        for pair in &self.0 {
            let Some((key_range, value_range)) = pair.as_ref() else {break};
            fmt += "`";
            fmt += buffer.read_str(key_range);
            fmt += "`: `";
            fmt += buffer.read_str(value_range);
            fmt += "`, ";
        }
        fmt + "]"
    }
}

pub struct RangeList {
    count: usize,
    list:  [Option<BufRange>; RANGE_COLLECTION_SIZE],
} impl RangeList {
    pub(crate) fn new() -> Self {
        Self {
            count: 0,
            list:  [None, None, None, None],
        }
    }
    pub(crate) fn push(&mut self, range: BufRange) -> Result<()> {
        (self.count < RANGE_COLLECTION_SIZE)
            ._else(|| Response::NotImplemented("Current ohkami can't handle more than 4 path params"))?;
        self.list[self.count] = Some(range);
        self.count += 1;
        Ok(())
    }
    pub(crate) fn get1(&self) -> Option<BufRange> {
        self.list.as_ref()[0]
    }
    pub(crate) fn get2(&self) -> Option<(BufRange, BufRange)> {
        let list = self.list.as_ref();
        Some((list[0]?, list[1]?))
    }
    // pub(crate) fn get3(&self) -> Option<(BufRange, BufRange, BufRange)> {
    //     let list = self.list.as_ref();
    //     Some((list[0]?, list[1]?, list[2]?))
    // }
    // pub(crate) fn get4(&self) -> Option<(BufRange, BufRange, BufRange, BufRange)> {
    //     let list = self.list.as_ref();
    //     Some((list[0]?, list[1]?, list[2]?, list[3]?))
    // }
}
