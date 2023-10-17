use super::{Request, METADATA_SIZE};
use crate::{__rt__, layer0_lib::{Slice, List, Method}};

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
    ";
    assert_parse!(case,
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
}
