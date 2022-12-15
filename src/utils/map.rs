use std::ops::Range;
use crate::{response::Response, result::{Result, ElseResponseWithErr}};
use super::hash::{TABLE_SIZE, linear_congruential_hash};

pub(crate) struct StringHashMap(
    [Option<String>; TABLE_SIZE]
); impl StringHashMap {
    pub fn new() -> Result<Self> {
        Ok(Self(
            TryInto::<[Option<String>; TABLE_SIZE]>::try_into(
                std::vec::from_elem(None, TABLE_SIZE)
            )._else(|_| Response::InternalServerError("Failed in type casting"))?
        ))
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.as_ref()
            .get(linear_congruential_hash(key) as usize)?
            .as_ref()
            .map(|string| &**string)
    }
    pub fn insert(&mut self, key: &str, value: String) {
        self.0[linear_congruential_hash(key) as usize] = Some(value)
    }
}

const RANGE_MAP_SIZE: usize = 4;
pub(crate) struct RangeMap(
    [(Range<usize>, Range<usize>); 4]
);