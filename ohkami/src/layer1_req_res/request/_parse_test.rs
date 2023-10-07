use super::{parse_formpart, parse_attachments};
use byte_reader::Reader;
use std::format as f;


#[test] fn test_parse_attachments() {
    const BOUNDARY: &str = "abcdef";

    let case = f!("\
        --{BOUNDARY}\r\n\
        Content-DIsposition: attachment; filename=\"file1.txt\"\r\n\
        \r\n\
        Hello, world!\r\n\
        --{BOUNDARY}--
    ");
    let mut r = Reader::new(case.as_bytes());
    assert_eq!(parse_attachments(&mut r, BOUNDARY), Ok(vec![]));
}
