use super::{parse_formpart, parse_attachments};
use byte_reader::Reader;
use std::format as f;


#[test] fn test_parse_attachments() {
    const BOUNDARY: &str = "abcdef";
    let mut r = Reader::from(f!("\
        --{BOUNDARY}\r\n\
        Content-DIsposition: attachment; filename=\"file1.txt\"\r\n\
        \r\n\
        Hello, world!\r\n\
        --{BOUNDARY}--
    "));
}
