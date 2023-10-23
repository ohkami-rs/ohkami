use super::{parse_formparts, parse_formpart, parse_attachments, parse_attachment, File, Field, FormPart, FormData};
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
        --{BOUNDARY}--\r\n\
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
    const BOUNDARY: &str = "AaB03x";

    let case = f!("\
        \r\n\
        Content-Disposition: form-data; name=\"field1\"\r\n\
        \r\n\
        Joe Blow\r\n\
        --{BOUNDARY}\
    "); assert_eq!(parse_formpart(&mut Reader::new(case.as_bytes()), BOUNDARY).unwrap(),
        Some((FormPart {
            name: f!("field1"),
            data: FormData::Field(Field {
                mime_type: Cow::Borrowed("text/plain"),
                content:   Vec::from("Joe Blow"),
            })
        }, false))
    );
}

#[test] fn test_parse_formparts() {
    let case = "\
        --AaB03x\r\n\
        Content-Disposition: form-data; name=\"field1\"\r\n\
        \r\n\
        Joe Blow\r\n\
        --AaB03x\r\n\
        content-type: application/json\r\n\
        content-DISPOSITION: form-data; name=\"user-data\"; filename=\"user.json\"\r\n\
        \r\n\
        {\r\n\
            \"name\":\"kanarus\",\r\n\
            \"age\":20\r\n\
        }\r\n\
        --AaB03x--\
    "; assert_eq!(parse_formparts(case.as_bytes(), "AaB03x").unwrap(), vec![
        FormPart {
            name: f!("field1"),
            data: FormData::Field(Field {
                mime_type: Cow::Borrowed("text/plain"),
                content:   Vec::from("Joe Blow"),
            })
        },
        FormPart {
            name: f!("user-data"),
            data: FormData::Files(vec![
                File {
                    name:      Some(f!("user.json")),
                    mime_type: Cow::Owned(f!("application/json")),
                    content:   Vec::from("\
                        {\r\n\
                            \"name\":\"kanarus\",\r\n\
                            \"age\":20\r\n\
                        }\
                    ")
                }
            ])
        }
    ]);

    let case = f!("\
        --AaB03x\r\n\
        Content-Disposition: form-data; name=\"field1\"\r\n\
        \r\n\
        Joe Blow\r\n\
        --AaB03x\r\n\
        content-type: multipart/mixed, boundary=BbC04y\r\n\
        content-DISPOSITION: form-data; name=\"pics\"\r\n\
        \r\n\
        --BbC04y\r\n\
        Content-DIsposition: attachment; filename=\"attachment1.txt\"\r\n\
        \r\n\
        << This is the content of `attachment1.txt` >>\r\n\
        --BbC04y\r\n\
        content-disposition: attachment; filename=\"attachment2.gif\"\r\n\
        Content-Transfer-Encoding: binary\r\n\
        Content-Type: image/gif\r\n\
        \r\n\
        [[binary\rbinary\rbinary\rbinary]]\r\n\
        --BbC04y--\r\n\
        --AaB03x\r\n\
        Content-Disposition: form-data; name=\"field2\"\r\n\
        Content-Type: text/html\r\n\
        \r\n\
        <p>Hello, world!</p>\r\n\
        --AaB03x--\
    "); assert_eq!(parse_formparts(case.as_bytes(), "AaB03x").unwrap(), vec![
        FormPart {
            name: f!("field1"),
            data: FormData::Field(Field {
                mime_type: Cow::Borrowed("text/plain"),
                content:   Vec::from("Joe Blow"),
            })
        },
        FormPart {
            name: f!("pics"),
            data: FormData::Files(vec![
                File {
                    name:      Some(f!("attachment1.txt")),
                    mime_type: Cow::Borrowed("text/plain"),
                    content:   Vec::from("<< This is the content of `attachment1.txt` >>")
                },
                File {
                    name:      Some(f!("attachment2.gif")),
                    mime_type: Cow::Owned(f!("image/gif")),
                    content:   Vec::from("[[binary\rbinary\rbinary\rbinary]]")
                },
            ])
        },
        FormPart {
            name: f!("field2"),
            data: FormData::Field(Field {
                mime_type: Cow::Owned(f!("text/html")),
                content:   Vec::from("<p>Hello, world!</p>"),
            })
        },
    ]);
}
