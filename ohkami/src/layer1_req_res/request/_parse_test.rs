use super::{parse_formpart, parse_attachments, parse_attachment, File};
use byte_reader::Reader;
use std::{format as f, borrow::Cow};


#[test] fn test_parse_attachment() {
    const BOUNDARY: &str = "abcdef";

    let case = f!("--");
    assert_eq!(parse_attachment(&mut Reader::new(case.as_bytes()), BOUNDARY).unwrap(), None);

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
            mime_type: Cow::Borrowed("text/plain"),
            content:   Vec::from("Hello, world!"),
        }, false))
    );

    let case = f!("\
        \r\n\
        Content-Disposition: attachment; filename=\"file2.html\"\r\n\
        Content-Type: text/html\r\n\
        \r\n\
        <h1>Hello, world!</h1>\r\n\
        --{BOUNDARY}--\r\n\
    ");
    assert_eq!(parse_attachment(&mut Reader::new(case.as_bytes()), BOUNDARY).unwrap(),
        Some((File {
            name:      Some(f!("file2.html")),
            mime_type: Cow::Borrowed("text/html"),
            content:   Vec::from("<h1>Hello, world!</h1>"),
        }, true))
    );

    let case = f!("\
        \r\n\
        Content-Type: application/json\r\n\
        Content-Disposition: attachment\r\n\
        \r\n\
        {{\r\n\
            \"name\":\"Alice\",\r\n\
            \"age\":14\r\n\
        }}\r\n\
        --{BOUNDARY}--\
    ");
    assert_eq!(parse_attachment(&mut Reader::new(case.as_bytes()), BOUNDARY).unwrap(),
        Some((File {
            name:      None,
            mime_type: Cow::Owned(f!("application/json")),
            content:   Vec::from("\
                {\r\n\
                    \"name\":\"Alice\",\r\n\
                    \"age\":14\r\n\
                }\
            "),
        }, true))
    );
}

#[test] fn test_parse_attachments() {
    const BOUNDARY: &str = "abcdef";

    let case = f!("\
        --{BOUNDARY}\r\n\
        Content-disposition: attachment\r\n\
        \r\n\
        Hello, world!\r\n\
        How are you?\r\n\
        --{BOUNDARY}\r\n\
        content-type: image/gif\r\n\
        Content-Disposition: attachment; filename=\"reaction.gif\"\r\n\
        Content-Transfer-Encoding: binary\r\n\
        \r\n\
        binarybinarybinarybinary\r\n\
        --{BOUNDARY}--\r\n\
    "); assert_eq!(parse_attachments(&mut Reader::new(case.as_bytes()), BOUNDARY).unwrap(),
        vec![
            File {
                name:      None,
                mime_type: Cow::Borrowed("text/plain"),
                content:   Vec::from("\
                    Hello, world!\r\n\
                    How are you?\
                ")
            },
            File {
                name:      Some(f!("reaction.gif")),
                mime_type: Cow::Owned(f!("image/gif")),
                content:   Vec::from("binarybinarybinarybinary")
            },
        ]
    );
}

#[test] fn test_parse_formpart() {
    todo!()
}
