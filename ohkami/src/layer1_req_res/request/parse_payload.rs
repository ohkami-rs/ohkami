use std::borrow::Cow;
use serde::Deserialize;


pub fn parse_json<'de, T: Deserialize<'de>>(buf: &'de [u8]) -> Result<T, Cow<'static, str>> {
    serde_json::from_slice(buf)
        .map_err(|e| Cow::Owned(e.to_string()))
}

// pub fn parse_form<T>(buf: &[u8]) -> Result<T, Cow<'static, str>> {
//     unimplemented!()
// }
// 
// pub fn parse_urlencoded<T>(buf: &[u8]) -> Result<T< Cow<'static, str>> {
//     
// }
// 