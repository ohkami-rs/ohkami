use std::pin::Pin;
use super::{Request, METADATA_SIZE, Path, QueryParams, Store};
use crate::{layer0_lib::{Slice, Method, CowSlice}};

macro_rules! assert_parse {
    ($case:expr, $expected:expr) => {
        let mut actual = Request::init();
        let mut actual = unsafe {Pin::new_unchecked(&mut actual)};
        actual.as_mut().read(&mut $case.as_bytes()).await;

        let expected = $expected;

        let _ = async {println!("<assert_parse>")}.await;

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

fn metadataize(input: &str) -> [u8; METADATA_SIZE] {
    let mut metadata = [0; METADATA_SIZE];
    metadata[..input.len().min(METADATA_SIZE)]
        .copy_from_slice(&input.as_bytes()[..input.len().min(METADATA_SIZE)]);
    metadata
}


#[crate::__rt__::test] async fn test_parse_request() {
    use super::{RequestHeader, RequestHeaders};

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
    assert_parse!(CASE_1, Request {_metadata: metadataize(CASE_1),
        method:  Method::GET,
        path:    Path::from_literal("/hello.html"),
        queries: QueryParams::new(),
        headers: RequestHeaders::from_iter([
            (RequestHeader::Host,           "www.tutorialspoint.com"),
            (RequestHeader::UserAgent,      "Mozilla/4.0"),
            (RequestHeader::Connection,     "Keep-Alive"),
            (RequestHeader::AcceptLanguage, "en-us"),
            (RequestHeader::AcceptEncoding, "gzip, deflate"),
        ]),
        payload: None,
        store:      Store::new(),
        #[cfg(feature="websocket")] upgrade_id: None,
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
    assert_parse!(CASE_2, Request {_metadata: metadataize(CASE_2),
        method:  Method::POST,
        path:    Path::from_literal("/signup"),
        queries: QueryParams::new(),
        headers: RequestHeaders::from_iter([
            (RequestHeader::Host,           "www.tutorialspoint.com"),
            (RequestHeader::UserAgent,      "Mozilla/4.0"),
            (RequestHeader::AcceptLanguage, "en-us"),
            (RequestHeader::ContentLength,  "27"),
            (RequestHeader::ContentType,    "application/json"),
        ]),
        payload: Some(CowSlice::Ref(unsafe {
            Slice::from_bytes(br#"{"name":"kanarus","age":20}"#)
        })),
        store:      Store::new(),
        #[cfg(feature="websocket")] upgrade_id: None,
    });


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
    assert_parse!(CASE_3, Request {_metadata: metadataize(CASE_3),
        method:  Method::POST,
        path:    Path::from_literal("/foo.php"),
        queries: QueryParams::from([
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
        payload: Some(CowSlice::Own(Vec::from("first_name=John&last_name=Doe&action=Submit"))),
        store:   Store::new(),
        #[cfg(feature="websocket")] upgrade_id: None,
    });
}
