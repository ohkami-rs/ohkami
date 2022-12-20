use crate::{result::{Result, ElseResponse}, response::Response};

use super::buffer::{Buffer, BufRange};
// use crate::{response::Response, result::{Result, ElseResponseWithErr}};
// use super::{hash::{TABLE_SIZE, linear_congruential_hash}, buffer::Buffer};
// 
// pub(crate) struct StringHashMap(
//     [Option<String>; TABLE_SIZE]
// ); impl StringHashMap {
//     pub fn new() -> Result<Self> {
//         Ok(Self(
//             TryInto::<[Option<String>; TABLE_SIZE]>::try_into(
//                 std::vec::from_elem(None, TABLE_SIZE)
//             )._else(|_| Response::InternalServerError("Failed in type casting"))?
//         ))
//     }
//     pub fn get(&self, key: &str) -> Option<&str> {
//         self.0.as_ref()
//             .get(linear_congruential_hash(key) as usize)?
//             .as_ref()
//             .map(|string| &**string)
//     }
//     pub fn insert(&mut self, key: &str, value: String) {
//         self.0[linear_congruential_hash(key) as usize] = Some(value)
//     }
// }


pub(crate) const RANGE_MAP_SIZE: usize = 4;
#[derive(Debug)]
pub(crate) struct RangeMap(
    [Option<(BufRange, BufRange)>; RANGE_MAP_SIZE]
); impl RangeMap {
    pub fn new() -> Self {
        Self([None, None, None, None])
    }
    pub fn insert(&mut self, index: usize, key: BufRange, value: BufRange) {
        self.0[index] = Some((key, value))
    }
    pub fn read_match_part_of_buffer<'map, 'key, 'buf>(
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

    pub fn debug_fmt_with(&self, buffer: &Buffer) -> String {
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


pub(crate) const STR_MAP_SIZE: usize = 4;
pub(crate) struct StrMap<'s> {
    count: usize,
    map:   [Option<(&'s str, &'s str)>; STR_MAP_SIZE]
} impl<'s> StrMap<'s> {
    pub fn new() -> Self {
        Self {
            count: 0,
            map:   [None, None, None, None]
        }
    }
    pub fn push(&mut self, key: &'s str, value: &'s str) -> Result<()> {
        (self.count == 4)
            ._else(|| Response::NotImplemented("Current ohkami can't handle more than 4 path params"))?;
        self.map[self.count] = Some((key, value));
        self.count += 1;
        Ok(())
    }
}