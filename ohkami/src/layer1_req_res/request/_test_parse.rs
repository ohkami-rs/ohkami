use super::{Request, METADATA_SIZE};
use crate::{__rt__, layer0_lib::{Slice, List, Method, ContentType}};

macro_rules! assert_parse {
    ($case:expr, $expected:expr) => {
        let actual   = Request::new(&mut $case.as_bytes()).await;
        let expected = $expected;
        if actual != expected {
            panic!("\n\
                =====  actual  =====\n\
                {actual:#?}\n\
                \n\
                ===== expected =====\n\
                {expected:#?}\n\
                \n\
            ")
        }
    };
}

fn metadataize(input: &str) -> [u8; METADATA_SIZE] {
    let input = input.as_bytes();
    assert!(input.len() <= METADATA_SIZE);

    let mut metadata = [0; METADATA_SIZE];
    metadata[..input.len()].copy_from_slice(input);
    metadata
}

#[__rt__::test] async fn test_parse_request() {
    let case = "\
        GET /hello.htm HTTP/1.1\r\n\
        User-Agent: Mozilla/4.0\r\n\
        Host: www.tutorialspoint.com\r\n\
        Accept-Language: en-us\r\n\
        Accept-Encoding: gzip, deflate\r\n\
        Connection: Keep-Alive\r\n\
        \r\n\
    "; assert_parse!(case,
        Request {_metadata: metadataize(case),
            method:  Method::GET,
            path:    unsafe {Slice::from_bytes(b"/hello.htm")},
            queries: List::from([]),
            headers: List::from([
                ("Host",            "www.tutorialspoint.com"),
                ("User-Agent",      "Mozilla/4.0"),
                ("Connection",      "Keep-Alive"),
                ("Accept-Language", "en-us"),
                ("Accept-Encoding", "gzip, deflate"),
            ]),
            payload: None,
        }
    );

    let case = "\
        POST /signup HTTP/1.1\r\n\
        User-Agent: Mozilla/4.0\r\n\
        Host: www.tutorialspoint.com\r\n\
        Accept-Language: en-us\r\n\
        Content-Type: application/json\r\n\
        Content-Length: 27\r\n\
        \r\n\
        {\"name\":\"kanarus\",\"age\":20}\
    "; assert_parse!(case,
        Request {_metadata: metadataize(case),
            method:  Method::POST,
            path:    unsafe {Slice::from_bytes(b"/signup")},
            queries: List::from([]),
            headers: List::from([
                ("Host",            "www.tutorialspoint.com"),
                ("User-Agent",      "Mozilla/4.0"),
                ("Accept-Language", "en-us"),
                ("Content-Length",  "27"),
                ("Content-Type",    "application/json"),
            ]),
            payload: Some((
                ContentType::JSON,
                Vec::from(r#"{"name":"kanarus","age":20}"#),
            )),
        }
    );

    let case = "\
        POST /foo.php HTTP/1.1\r\n\
        Host: localhost\r\n\
        User-Agent: Mozilla/5.0 (Windows; U; Windows NT 6.1; en-US; rv:1.9.1.5) Gecko/20091102 Firefox/3.5.5 (.NET CLR 3.5.30729)\r\n\
        Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\
        Accept-Language: en-us,en;q=0.5\r\n\
        Accept-Encoding: gzip,deflate\r\n\
        Accept-Charset: ISO-8859-1,utf-8;q=0.7,*;q=0.7\r\n\
        Keep-Alive: 300\r\n\
        Connection: keep-alive\r\n\
        Referer: http://localhost/test.php\r\n\
        Content-Type: application/x-www-form-urlencoded\r\n\
        Content-Length: 43\r\n\
        \r\n\
        first_name=John&last_name=Doe&action=Submit\
    ";
}
