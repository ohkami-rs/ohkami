//! Format of a part of request/response
//! 
//! ## Builtin
//! 
//! - `Query` - query parameters
//! - `JSON` - payload of application/json
//! - `Multipart` - payload of multipart/form-data
//! - `URLEncoded` - payload of application/x-www-form-urlencoded
//! - `Text` - payload of text/plain
//! - `HTML` - payload of text/html

mod builtin;
pub use builtin::*;
