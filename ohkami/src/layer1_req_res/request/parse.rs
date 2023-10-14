use byte_reader::Reader;

use super::{Request, QUERIES_LIMIT, HEADERS_LIMIT};
use crate::layer0_lib::{Buffer, Method, List, ContentType, Slice};


pub(super) fn parse(buffer: Buffer) -> Request {
    let mut r = Reader::new(buffer.as_bytes());

    let method = Method::from_bytes(r.read_while(|b| b != &b' '));
    r.consume(" ").unwrap();
    
    let path = unsafe {Slice::from_bytes(r.read_while(|b| b != &b'?' && b != &b' '))};

    let mut queries = List::<_, {QUERIES_LIMIT}>::new();
    if r.consume_oneof([" ", "?"]).unwrap() == 1 {
        while r.peek().is_some() {
            let key = unsafe {Slice::from_bytes(r.read_while(|b| b != &b'='))};
            r.consume("=").unwrap();
            let val = unsafe {Slice::from_bytes(r.read_while(|b| b != &b'&' && b != &b' '))};

            queries.append((key, val));
            if r.consume_oneof(["&", " "]).unwrap() == 1 {break}
        }
    }

    r.consume("HTTP/1.1\r\n").expect("Ohkami can only handle HTTP/1.1");

    let mut headers      = List::<_, {HEADERS_LIMIT}>::new();
    let mut content_type = None;
    while r.consume("\r\n").is_none() {
        let key = unsafe {Slice::from_bytes(r.read_while(|b| b != &b':'))};
        r.consume(": ").unwrap();
        let val = unsafe {Slice::from_bytes(r.read_while(|b| b != &b'\r'))};
        r.consume("\r\n").unwrap();

        headers.append((key, val));
        b"Content-Type".eq_ignore_ascii_case(unsafe {key.into_bytes()})
            .then(|| content_type = ContentType::from_bytes(unsafe {val.into_bytes()}));
    }

    let payload = r.peek().is_some().then(|| (
        content_type.unwrap_or(ContentType::Text),
        unsafe {Slice::from_bytes(r.read_while(|_| true))}
    ));

    Request { _buffer:buffer, method, path, queries, headers, payload }
}




#[cfg(test)] #[test]
fn check_request_parsing() {
    use super::DebugRequest;

    DebugRequest {
        method: Method::GET,
        path: "/hello.htm",
        queries: &[],
        headers: &[
            ("User-Agent", "Mozilla/4.0 (compatible; MSIE5.01; Windows NT)"),
            ("Host", "www.tutorialspoint.com"),
            ("Accept-Language", "en-us"),
            ("Accept-Encoding", "gzip, deflate"),
            ("Connection", "Keep-Alive"),
        ],
        payload: None
    }.assert_parsed_from("\
        GET /hello.htm HTTP/1.1\r\n\
        User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\n\
        Host: www.tutorialspoint.com\r\n\
        Accept-Language: en-us\r\n\
        Accept-Encoding: gzip, deflate\r\n\
        Connection: Keep-Alive\r\n\
        \r\n\
    ");

    DebugRequest {
        method: Method::POST,
        path: "/cgi-bin/process.cgi",
        queries: &[],
        headers: &[
            ("User-Agent", "Mozilla/4.0 (compatible; MSIE5.01; Windows NT)"),
            ("Host", "www.tutorialspoint.com"),
            // ("Content-Type", "application/x-www-form-urlencoded"),
            ("Content-Length", "length"),
            ("Accept-Language", "en-us"),
            ("Accept-Encoding", "gzip, deflate"),
            ("Connection", "Keep-Alive")
        ],
        payload: Some((
            ContentType::URLEncoded,
            "licenseID=string&content=string&/paramsXML=string"
        )),
    }.assert_parsed_from("\
        POST /cgi-bin/process.cgi HTTP/1.1\r\n\
        User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\n\
        Host: www.tutorialspoint.com\r\n\
        Content-Type: application/x-www-form-urlencoded\r\n\
        Content-Length: length\r\n\
        Accept-Language: en-us\r\n\
        Accept-Encoding: gzip, deflate\r\n\
        Connection: Keep-Alive\r\n\
        \r\n\
        licenseID=string&content=string&/paramsXML=string\
    ");

    DebugRequest {
        method: Method::GET,
        path: "/genapp/customers",
        queries: &[
            ("name", "Joe Bloggs"),
            ("email", "abc@email.com"),
        ],
        headers: &[
            ("Host", "www.example.com")
        ],
        payload: None,
    }.assert_parsed_from("\
        GET /genapp/customers?name=Joe%20Bloggs&email=abc@email.com HTTP/1.1\r\n\
        Host: www.example.com\r\n\
        \r\n\
    ");
}
