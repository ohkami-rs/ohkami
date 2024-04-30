use crate::header::append;
use super::ResponseHeaders;


#[test] fn append_header() {
    let mut h = ResponseHeaders::new();

    h.set().Server(append("X"));
    assert_eq!(h.Server(), Some("X"));
    h.set().Server(append("Y"));
    assert_eq!(h.Server(), Some("X,Y"));
}

#[test] fn append_custom_header() {
    let mut h = ResponseHeaders::new();

    h.set().custom("Custom-Header", append("A"));
    assert_eq!(h.custom("Custom-Header"), Some("A"));
    h.set().custom("Custom-Header", append("B"));
    assert_eq!(h.custom("Custom-Header"), Some("A,B"));
}
