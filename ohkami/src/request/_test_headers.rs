#![cfg(any(feature="testing", feature="DEBUG"))]
#![cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]

use ohkami_lib::CowSlice;

use super::{RequestHeader, RequestHeaders};
use crate::header::append;


#[test] fn append_header() {
    let mut h = RequestHeaders::init();

    h.append(RequestHeader::Origin, CowSlice::from("A".as_bytes()));
    assert_eq!(h.Origin(), Some("A"));
    h.append(RequestHeader::Origin, CowSlice::from("B".as_bytes()));
    assert_eq!(h.Origin(), Some("A,B"));

    h.set().Accept(append("X"));
    assert_eq!(h.Accept(), Some("X"));
    h.set().Accept(append("Y"));
    assert_eq!(h.Accept(), Some("X,Y"));
}

#[test] fn append_custom_header() {
    let mut h = RequestHeaders::init();

    h.set().custom("Custom-Header", append("A"));
    assert_eq!(h.custom("Custom-Header"), Some("A"));
    h.set().custom("Custom-Header", append("B"));
    assert_eq!(h.custom("Custom-Header"), Some("A,B"));
}
