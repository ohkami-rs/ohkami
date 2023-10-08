use super::{parse_formpart, parse_attachments, parse_attachment, File};
use byte_reader::Reader;
use std::format as f;


#[test] fn test_parse_attachment() {
    const BOUNDARY: &str = "abcdef";

    let case = f!("\
        \r\n\
        Content-Disposition: attachment; filename=\"file1.txt\"\r\n\
        \r\n\
        Hello, world!\r\n\
        --{BOUNDARY}\r\n\
    ");
    assert_eq!(parse_attachment(&mut Reader::new(case.as_bytes()), BOUNDARY).unwrap(),
        Some((File {
            name:      Some(f!("file1.txt")),
            mime_type: f!("text/plain"),
            content:   Vec::from("Hello, world!"),
        }, false))
    );
}
