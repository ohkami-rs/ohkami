#![cfg(all(test, feature="__rt_native__", feature="DEBUG"))]

#[allow(unused)]
use super::{Request, Method, Path, QueryParams, Context};

use super::{RequestHeader, RequestHeaders};
use std::pin::Pin;
use ohkami_lib::{Slice, CowSlice};

#[test]
fn parse_path() {
    let mut path = Path::uninit();
    path.init_with_request_bytes(b"/abc").unwrap();
    assert_eq!(&*path, "/abc");

    let mut path = Path::uninit();
    path.init_with_request_bytes(b"/abc/").unwrap();
    assert_eq!(&*path, "/abc");

    let mut path = Path::uninit();
    path.init_with_request_bytes(b"/").unwrap();
    assert_eq!(&*path, "/");
}

macro_rules! assert_parse {
    ($case:expr, $expected:expr) => {
        let mut case = $case.as_bytes();

        let mut actual = Request::init(crate::util::IP_0000);
        let mut actual = unsafe {Pin::new_unchecked(&mut actual)};
        
        let result = crate::__rt__::testing::block_on({
            actual.as_mut().read(&mut case)
        });

        assert_eq!(result, Ok(Some(())));
        
        let expected = $expected;

        println!("<assert_parse>");

        let __panic_message = format!("\n\
            =====  actual  =====\n\
            {actual:#?}\n\
            \n\
            ===== expected =====\n\
            {expected:#?}\n\
            \n\
        ");

        if actual.get_mut() != &expected {
            panic!("{__panic_message}")
        }
    };
}

fn metadataize(input: &str) -> Box<[u8]> {
    let buf_size = crate::CONFIG.request_bufsize();
    let mut buf = vec![0; buf_size];
    buf[..input.len().min(buf_size)]
        .copy_from_slice(&input.as_bytes()[..input.len().min(buf_size)]);
    buf.into_boxed_slice()
}

#[test] fn test_parse_request() {
    const CASE_1: &str = "\
        GET /hello.html HTTP/1.1\r\n\
        User-Agent: Mozilla/4.0\r\n\
        Host: www.tutorialspoint.com\r\n\
        Accept-Language: en-us\r\n\
        Accept-Encoding: gzip, deflate\r\n\
        Connection: Keep-Alive\r\n\
        \r\n\
    ";
    const _CASE_1_LEN: usize = CASE_1.len();
    assert_parse!(CASE_1, Request {
        __buf__: metadataize(CASE_1),
        method:  Method::GET,
        path:    Path::from_literal("/hello.html"),
        query:   QueryParams::new(b""),
        headers: RequestHeaders::from_iters([
            (RequestHeader::Host,           "www.tutorialspoint.com"),
            (RequestHeader::UserAgent,      "Mozilla/4.0"),
            (RequestHeader::Connection,     "Keep-Alive"),
            (RequestHeader::AcceptLanguage, "en-us"),
            (RequestHeader::AcceptEncoding, "gzip, deflate"),
        ], None),
        payload: None,
        context: Context::init(),
        ip:      crate::util::IP_0000
    });


    const CASE_2: &str = "\
        POST /signup HTTP/1.1\r\n\
        User-Agent: Mozilla/4.0\r\n\
        Host: www.tutorialspoint.com\r\n\
        Accept-Language: en-us\r\n\
        Content-Type: application/json\r\n\
        Content-Length: 27\r\n\
        \r\n\
        {\"name\":\"kanarus\",\"age\":20}\
    ";
    const _CASE_2_LEN: usize = CASE_2.len();
    assert_parse!(CASE_2, Request {
        __buf__: metadataize(CASE_2),
        method:  Method::POST,
        path:    Path::from_literal("/signup"),
        query:   QueryParams::new(b""),
        headers: RequestHeaders::from_iters([
            (RequestHeader::Host,           "www.tutorialspoint.com"),
            (RequestHeader::UserAgent,      "Mozilla/4.0"),
            (RequestHeader::AcceptLanguage, "en-us"),
            (RequestHeader::ContentLength,  "27"),
            (RequestHeader::ContentType,    "application/json"),
        ], None),
        payload: Some(CowSlice::Ref(Slice::from_bytes(
            br#"{"name":"kanarus","age":20}"#
        ))),
        context: Context::init(),
        ip:      crate::util::IP_0000
    });

    {
        const CASE_3: &str = "\
            POST /foo.php?query=1&q2=xxx HTTP/1.1\r\n\
            Host: localhost\r\n\
            User-Agent: Mozilla/5.0 (Windows; U; Windows NT 6.1; en-US; rv:1.9.1.5) Gecko/20091102 Firefox/3.5.5 (.NET CLR 3.5.30729)\r\n\
            Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
            Accept-Language: en-us,en;q=0.5\r\n\
            Accept-Encoding: gzip,deflate\r\n\
            X-Request-Id: 300\r\n\
            Connection: keep-alive\r\n\
            Referer: http://localhost/test.php\r\n\
            Content-Type: application/x-www-form-urlencoded\r\n\
            Content-Length: 43\r\n\
            \r\n\
            first_name=John&last_name=Doe&action=Submit\
        ";
        const _CASE_3_LEN: usize = CASE_3.len();
        assert_parse!(CASE_3, Request {
            __buf__: metadataize(CASE_3),
            method:  Method::POST,
            path:    Path::from_literal("/foo.php"),
            query:   QueryParams::from([
                ("query", "1"),
                ("q2",    "xxx"),
            ]),
            headers: RequestHeaders::from_iters(
                [
                    (RequestHeader::Host,           "localhost"),
                    (RequestHeader::UserAgent,      "Mozilla/5.0 (Windows; U; Windows NT 6.1; en-US; rv:1.9.1.5) Gecko/20091102 Firefox/3.5.5 (.NET CLR 3.5.30729)"),
                    (RequestHeader::Accept,         "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
                    (RequestHeader::AcceptLanguage, "en-us,en;q=0.5"),
                    (RequestHeader::AcceptEncoding, "gzip,deflate"),
                    (RequestHeader::Connection,     "keep-alive"),
                    (RequestHeader::Referer,        "http://localhost/test.php"),
                    (RequestHeader::ContentType,    "application/x-www-form-urlencoded"),
                    (RequestHeader::ContentLength,  "43"),
                ],
                [
                    ("X-Request-Id", "300"),
                ]
            ),
            payload: Some(CowSlice::Own(Vec::from("first_name=John&last_name=Doe&action=Submit").into())),
            context: Context::init(),
            ip:      crate::util::IP_0000
        });
    }
}
