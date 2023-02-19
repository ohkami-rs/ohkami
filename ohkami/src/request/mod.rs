mod from_request;
pub(crate) mod parse;

use std::{collections::BTreeMap, str::Split};
use crate::error::Error;


pub struct Request<'buf> {
    pub path_params:  RequestPathParams<'buf>,
    pub query_params: RequestQueryParams<'buf>,
    pub headers:      RequestHeaders<'buf>,
    pub body:         Option<&'buf str>,
}


const PATH_PARAMS_LIMIT: usize = 2;
pub struct RequestPathParams<'buf>{
    params:   [Option<&'buf str>; PATH_PARAMS_LIMIT],
    next_pos: usize,
} const _: (/* RequestPathParams impls */) = {
    impl<'buf> RequestPathParams<'buf> {
        #[inline] pub(crate) fn new(path_str: &'buf str) -> Self {
            Self(
                path_str
                    .trim_end_matches('/')
                    .split('/')
            )
        }
    }
};

const QUERY_PARAMS_LIMIT: usize = 4;
pub struct RequestQueryParams<'buf> {
    keys:   [Option<&'buf str>; QUERY_PARAMS_LIMIT],
    values: [Option<&'buf str>; QUERY_PARAMS_LIMIT],
    next_pos: usize,
} const _: (/* RequestQueryParams impls */) = {
    impl<'buf> RequestQueryParams<'buf> {
        pub(crate) fn new() -> Self {
            Self {
                keys:   [None, None, None, None],
                values: [None, None, None, None],
                next_pos: 0,
            }
        }
        pub(crate) fn insert(&mut self, key: &'buf str, value: &'buf str) -> crate::Result<()> {
            if self.next_pos == QUERY_PARAMS_LIMIT {
                return Err(Error::in_const_value("QUERY_PARAMS_LIMIT"))
            }

            self.keys[self.next_pos].replace(key);
            self.values[self.next_pos].replace(value);
            self.next_pos += 1;

            Ok(())
        }
    }
};

pub struct RequestHeaders<'buf>(
    BTreeMap<&'buf str, &'buf str>
); const _: (/* RequestHeader impls */) = {
    impl<'buf> RequestHeaders<'buf> {
        #[inline] pub(crate) fn new() -> Self {
            Self(BTreeMap::new())
        }
        #[inline] pub(crate) fn insert(&mut self, (key, value): (&'buf str, &'buf str)) {
            self.0.insert(key, value)
        }
    }
};
