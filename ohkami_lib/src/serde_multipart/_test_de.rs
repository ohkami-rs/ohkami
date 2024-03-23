use super::{from_bytes, File};


#[test] fn deserialize_single() {
    const BOUNDARY: &str = "AaB03x";

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct UserForm<'req> {
        #[serde(rename = "user-name")]
        user_name: &'req str,
    }
    let case = format!("\
        --{BOUNDARY}\r\n\
        Content-Disposition: form-data; name=\"user-name\"\r\n\
        \r\n\
        Joe Blow\r\n\
        --{BOUNDARY}--");
    assert_eq!(
        from_bytes::<UserForm>(case.as_bytes()).unwrap(),
        UserForm {
            user_name: "Joe Blow",
        }
    );

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct FilesForm<'req> {
        #[serde(borrow)]
        files: File<'req>,
    }
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
        from_bytes::<FilesForm>(case.as_bytes()).unwrap(),
        FilesForm {
            files: File {
                filename: "test.md",
                mimetype: "text/markdown",
                content:  b"\
                # Why ohkami\n\
                \n\
                `ohkami` is a newbee Rust web framework with features:\n\
                \n\
                - (...TODO...)\n",
            }
        }
    );
}

#[test] fn deserialize_multiple_in_one_name_one_item() {
    const BOUNDARY: &str = "Bbax09y";

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct UserTemplateForm<'req> {
        #[serde(rename = "user-name")]
        user_name: &'req str,
        template:  File<'req>,
    }
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
        from_bytes::<UserTemplateForm>(case.as_bytes()).unwrap(),
        UserTemplateForm {
            user_name: "Mr. admin\r\n(hmm...)",
            template:  File {
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
        }
    );
}

#[test] fn deserialize_multiple() {
    const BOUNDARY: &str = "Bbax09y";

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct UserFilesForm<'req> {
        #[serde(rename = "user-name")]
        user_name:     &'req str,

        templates:     Vec<File<'req>>,

        #[serde(rename = "binary-sample")]
        binary_sample: File<'req>,
    }
    let case = format!("\
        --{BOUNDARY}\r\n\
        Content-Disposition: form-data; name=\"user-name\"\r\n\
        \r\n\
        Hi, Mr. admin\r\n\
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
        Content-Disposition: form-data; name=\"binary-sample\"; filename=\"x.bin\"\r\n\
        Something-Unknown-Header: unknown-header-value\r\n\
        \r\n\
        \r\u{0}\r\u{0}\n0123xyz\u{11}\r\n\u{10}\rabc\r\n\
        --{BOUNDARY}--");
    assert_eq!(
        from_bytes::<UserFilesForm>(case.as_bytes()).unwrap(),
        UserFilesForm {
            user_name: "Hi, Mr. admin",
            templates: vec![
                File {
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
                File {
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
                    </html>\n".as_bytes()
                },
            ],
            binary_sample: File {
                filename: "x.bin",
                mimetype: "unknown/some-binary",
                content:  "\
                \r\u{0}\r\u{0}\n0123xyz\u{11}\r\n\u{10}\rabc".as_bytes(),
            },
        }
    );
}

#[test] fn deserialize_optionals() {
    const BOUNDARY: &str = "Bbax09y";

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct UserFilesForm<'req> {
        #[serde(rename = "user-name")]
        user_name:     Option<&'req str>,

        templates:     Vec<File<'req>>,

        #[serde(rename = "binary-sample")]
        binary_sample: Option<File<'req>>,
    }

    let case_1 = format!("\
        --{BOUNDARY}\r\n\
        Content-Disposition: form-data; name=\"user-name\"\r\n\
        \r\n\
        Jacky\r\n\
        --{BOUNDARY}\r\n\
        content-type: application/octet-stream\r\n\
        CONTENT-DISPOSITION: form-data; name=\"templates\"; filename=\"\"\r\n\
        \r\n\
        \r\n\
        --{BOUNDARY}\r\n\
        Content-Type: unknown/some-binary\r\n\
        Content-Disposition: form-data; name=\"binary-sample\"; filename=\"x.bin\"\r\n\
        Something-Unknown-Header: unknown-header-value\r\n\
        \r\n\
        \r\u{0}\r\u{0}\n0123xyz\u{11}\r\n\u{10}\rabc\r\n\
        --{BOUNDARY}--");
    assert_eq!(
        from_bytes::<UserFilesForm>(case_1.as_bytes()).unwrap(),
        UserFilesForm {
            user_name: Some("Jacky"),
            templates: vec![],
            binary_sample: Some(File {
                filename: "x.bin",
                mimetype: "unknown/some-binary",
                content:  "\r\u{0}\r\u{0}\n0123xyz\u{11}\r\n\u{10}\rabc".as_bytes(),
            }),
        }
    );

    let case_2 = format!("\
        --{BOUNDARY}\r\n\
        Content-Disposition: form-data; name=\"user-name\"\r\n\
        \r\n\
        \r\n\
        --{BOUNDARY}\r\n\
        content-type: text/html\r\n\
        CONTENT-DISPOSITION: form-data; name=\"templates\"; filename=\"tiny.html\"\r\n\
        \r\n\
        <html>\n\
        <h1>Tiny Document</h1>\n\
        <p>Hi, this is composed of one h1 and one p tag!</p>\n\
        </html>\n\
        \r\n\
        --{BOUNDARY}\r\n\
        Content-Type: unknown/some-binary\r\n\
        Content-Disposition: form-data; name=\"binary-sample\"; filename=\"x.bin\"\r\n\
        Something-Unknown-Header: unknown-header-value\r\n\
        \r\n\
        \r\u{0}\r\u{0}\n0123xyz\u{11}\r\n\u{10}\rabc\r\n\
        --{BOUNDARY}--");
    assert_eq!(
        from_bytes::<UserFilesForm>(case_2.as_bytes()).unwrap(),
        UserFilesForm {
            user_name: None,
            templates: vec![
                File {
                    filename: "tiny.html",
                    mimetype: "text/html",
                    content: b"\
                    <html>\n\
                    <h1>Tiny Document</h1>\n\
                    <p>Hi, this is composed of one h1 and one p tag!</p>\n\
                    </html>\n"
                }
            ],
            binary_sample: Some(File {
                filename: "x.bin",
                mimetype: "unknown/some-binary",
                content:  "\r\u{0}\r\u{0}\n0123xyz\u{11}\r\n\u{10}\rabc".as_bytes(),
            }),
        }
    );
}
