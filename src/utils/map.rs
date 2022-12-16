use std::ops::RangeInclusive;
use super::buffer::Buffer;
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

#[derive(Clone, Copy)]
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


pub(crate) const RANGE_MAP_SIZE: usize = 4;
pub(crate) struct RangeMap(
    [Option<(BufRange, BufRange)>; 4]
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
}
