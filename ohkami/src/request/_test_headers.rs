#![cfg(any(feature="testing", feature="DEBUG"))]
#![cfg(any(feature="rt_tokio",feature="rt_async-std",feature="rt_worker"))]

use std::borrow::Cow;
use super::{RequestHeader, RequestHeaders};
use crate::append;


#[test] fn append_header() {
    let mut h = RequestHeaders::init();

    h.append(RequestHeader::Origin, Cow::Borrowed("A"));
    assert_eq!(h.Origin(), Some("A"));
    h.append(RequestHeader::Origin, Cow::Borrowed("B"));
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
