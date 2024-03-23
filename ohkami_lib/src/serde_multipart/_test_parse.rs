use super::parse::{Multipart, Part};
use super::File;


#[test] fn parse_single() {
    const BOUNDARY: &str = "AaB03x";

    let case = format!("\
        --{BOUNDARY}\r\n\
        Content-Disposition: form-data; name=\"user-name\"\r\n\
        \r\n\
        Joe Blow\r\n\
        --{BOUNDARY}--");
    assert_eq!(
        Multipart::parse(case.as_bytes()).unwrap(),
        Multipart(vec![
            Part::Text { name: "user-name", text: "Joe Blow" },
        ])
    );

    let case = format!("\
        --{BOUNDARY}\r\n\
        Content-Disposition: form-data; name=\"files\"; filename=\"test.md\"\r\n\
        content-type: text/markdown\r\n\
        \r\n\
        # Why ohkami\n\
        \n\
        `ohkami` is a newbee Rust web framework with features:\n\
        \n\
        - (...TODO...)\n\
        \r\n--{BOUNDARY}--");
    assert_eq!(
        Multipart::parse(case.as_bytes()).unwrap(),
        Multipart(vec![
            Part::File {
                name: "files",
                file: File {
                    filename: "test.md",
                    mimetype: "text/markdown",
                    content:  "\
                        # Why ohkami\n\
                        \n\
                        `ohkami` is a newbee Rust web framework with features:\n\
                        \n\
                        - (...TODO...)\n".as_bytes(),
                }
            },
        ])
    );
}

#[test] fn parse_multiple_in_one_name_one_item() {
    const BOUNDARY: &str = "Bbax09y";

    let case = format!("\
        --{BOUNDARY}\r\n\
        Content-Disposition: form-data; name=\"user-name\"\r\n\
        \r\n\
        Mr. admin\r\n\
        (hmm...)\r\n\
        --{BOUNDARY}\r\n\
        content-type: text/html\r\n\
        CONTENT-DISPOSITION: form-data; name=\"template\"; filename=\"index.html\"\r\n\
        \r\n\
        <!DOCTYPE html>\n\
        <html lang=\"en-US\">\n\
        <head>\n\
        <title>Document</title>\n\
        </head>\n\
        <body>\n\
        <p>Hello, this is a test case！</p>\n\
        </body>\n\
        </html>\
        \r\n--{BOUNDARY}--");
    assert_eq!(
        Multipart::parse(case.as_bytes()).unwrap(),
        Multipart(vec![
            Part::Text {
                name: "user-name",
                text: "Mr. admin\r\n(hmm...)",
            },
            Part::File {
                name: "template",
                file: File {
                    filename: "index.html",
                    mimetype: "text/html",
                    content:  "\
                    <!DOCTYPE html>\n\
                    <html lang=\"en-US\">\n\
                    <head>\n\
                    <title>Document</title>\n\
                    </head>\n\
                    <body>\n\
                    <p>Hello, this is a test case！</p>\n\
                    </body>\n\
                    </html>".as_bytes(),
                },
            },
        ])
    );
}

#[test] fn parse_multiple() {
    const BOUNDARY: &str = "Bbax09y";

    let case = format!("\
        --{BOUNDARY}\r\n\
        Content-Disposition: form-data; name=\"user-name\"\r\n\
        \r\n\
        Mr. admin\r\n\
        (hmm...)\r\n\
        --{BOUNDARY}\r\n\
        content-type: text/html\r\n\
        CONTENT-DISPOSITION: form-data; name=\"templates\"; filename=\"index.html\"\r\n\
        \r\n\
        <!DOCTYPE html>\n\
        <html lang=\"en-US\">\n\
        <head>\n\
        <title>Document</title>\n\
        </head>\n\
        <body>\n\
        <p>Hello, this is a test case！</p>\n\
        </body>\n\
        </html>\r\n\
        --{BOUNDARY}\r\n\
        Content-Disposition: form-data; name=\"templates\"; filename=\"home.html\"\r\n\
        Content-Type: text/html\r\n\
        \r\n\
        <!DOCTYPE html>\n\
        <html lang=\"en-US\">\n\
        <head>\n\
        <style>\n\
        h1 {{\n\
        color: red;\n\
        }}\n\
        </style>\n\
        <title>Home</title>\n\
        </head>\n\
        <body>\n\
        <h1>This is HOME page.</h1>\n\
        </body>\n\
        </html>\n\
        \r\n\
        --{BOUNDARY}\r\n\
        Content-Type: unknown/some-binary\r\n\
        Content-Disposition: form-data; name=\"binary_sample\"; filename=\"x.bin\"\r\n\
        Something-Unknown-Header: unknown-header-value\r\n\
        \r\n\
        \r\u{0}\r\u{0}\n0123xyz\u{11}\r\n\u{10}\rabc\r\n\
        --{BOUNDARY}--");
    assert_eq!(
        Multipart::parse(case.as_bytes()).unwrap(),
        Multipart(vec![
            Part::Text {
                name: "user-name",
                text: "Mr. admin\r\n(hmm...)",
            },

            Part::File {
                name: "templates",
                file: File {
                    filename: "index.html",
                    mimetype: "text/html",
                    content:  "\
                    <!DOCTYPE html>\n\
                    <html lang=\"en-US\">\n\
                    <head>\n\
                    <title>Document</title>\n\
                    </head>\n\
                    <body>\n\
                    <p>Hello, this is a test case！</p>\n\
                    </body>\n\
                    </html>".as_bytes()
                },
            },
            Part::File {
                name: "templates",
                file: File {
                    filename: "home.html",
                    mimetype: "text/html",
                    content:  "\
                    <!DOCTYPE html>\n\
                    <html lang=\"en-US\">\n\
                    <head>\n\
                    <style>\n\
                    h1 {\n\
                    color: red;\n\
                    }\n\
                    </style>\n\
                    <title>Home</title>\n\
                    </head>\n\
                    <body>\n\
                    <h1>This is HOME page.</h1>\n\
                    </body>\n\
                    </html>\n".as_bytes(),
                },
            },

            Part::File {
                name: "binary_sample",
                file: File {
                    filename: "x.bin",
                    mimetype: "unknown/some-binary",
                    content:  "\r\u{0}\r\u{0}\n0123xyz\u{11}\r\n\u{10}\rabc".as_bytes(),
                },
            },
        ])
    );
}
