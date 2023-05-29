use super::{Request, QUERIES_LIMIT, HEADERS_LIMIT};
use crate::layer0_lib::{Buffer, Method, List};


pub(super) fn parse(buffer: Buffer) -> Request {
    let mut start = 0;

    let method = {
        let mut end = start;
        for b in &buffer[start..] {
            match b {
                b' ' => break,
                _ => end += 1,
            }
        }
        let method = Method::from_bytes(&buffer[start..end]);
        start = end + 1;
        method
    };

    let mut includes_queries = false;
    let path = {
        let mut end = start;
        for b in &buffer[start..] {
            match b {
                b'?' => {includes_queries = true; break}
                b' ' => break,
                _ => end += 1,
            }
        }
        let path = start..end;
        start = end + 1;
        path
    };

    let mut queries = List::<_, {QUERIES_LIMIT}>::new(); if includes_queries {
        let mut query_start = start;
        loop {
            let mut is_final = false;

            let mut eq = query_start;
            for b in &buffer[query_start..] {
                match *b {
                    b'=' => break,
                    _    => eq += 1,
                }
            }

            let mut end = eq + 1;
            for b in &buffer[end..] {
                match b {
                    b' ' => {is_final = true; break},
                    b'&' => break,
                    _ => end += 1,
                }
            }

            queries.append((
                query_start..eq,
                (eq+1)..end,
            ));
            query_start = end + 1/* ' ' or '&' */;
            if is_final {break}
        }
        start = query_start
    }

    let _/* HTTP version */ = {
        for b in &buffer[start..] {
            start += 1;
            if *b == b'\n' {break}
        }
    };

    let mut headers = List::<_, {HEADERS_LIMIT}>::new(); {
        let mut header_start = start;
        loop {
            match buffer[header_start] {
                b'\0' | b'\r' => break,
                _ => (),
            }

            let mut colon = header_start;
            for b in &buffer[header_start..] {
                match b {
                    b':' => break,
                    _ => colon += 1,
                }
            }

            let mut end = colon + 1/* ' ' */ + 1;
            for b in &buffer[end..] {
                match b {
                    b'\0' | b'\r' => break,
                    _ => end += 1,
                }
            }

            headers.append((
                header_start..colon,
                (colon+1/* ' ' */+1)..end,
            ));
            header_start = end + 1/* '\n' */ + 1
        }
        start = header_start + 1/* '\n' */ + 1
    };

    let body = (buffer[start] != 0).then(|| {
        let mut end = start;
        for b in &buffer[start..] {
            match b {
                b'\0' => break,
                _ => end += 1,
            }
        }
        start..end
    });

    Request {
        buffer,
        method,
        path,
        queries,
        headers,
        body
    }
}




#[cfg(test)]#[test]
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
        body: None
    }.assert_parsed_from(
"GET /hello.htm HTTP/1.1\r
User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r
Host: www.tutorialspoint.com\r
Accept-Language: en-us\r
Accept-Encoding: gzip, deflate\r
Connection: Keep-Alive"
    );

    DebugRequest {
        method: Method::POST,
        path: "/cgi-bin/process.cgi",
        queries: &[],
        headers: &[
            ("User-Agent", "Mozilla/4.0 (compatible; MSIE5.01; Windows NT)"),
            ("Host", "www.tutorialspoint.com"),
            ("Content-Type", "application/x-www-form-urlencoded"),
            ("Content-Length", "length"),
            ("Accept-Language", "en-us"),
            ("Accept-Encoding", "gzip, deflate"),
            ("Connection", "Keep-Alive")
        ],
        body: Some("licenseID=string&content=string&/paramsXML=string"),
    }.assert_parsed_from(
"POST /cgi-bin/process.cgi HTTP/1.1\r
User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r
Host: www.tutorialspoint.com\r
Content-Type: application/x-www-form-urlencoded\r
Content-Length: length\r
Accept-Language: en-us\r
Accept-Encoding: gzip, deflate\r
Connection: Keep-Alive\r
\r
licenseID=string&content=string&/paramsXML=string"
    );

    DebugRequest {
        method: Method::GET,
        path: "/genapp/customers",
        queries: &[
            ("name", "Joe%20Bloggs"),
            ("email", "abc@email.com"),
        ],
        headers: &[
            ("Host", "www.example.com")
        ],
        body: None,
    }.assert_parsed_from(
"GET /genapp/customers?name=Joe%20Bloggs&email=abc@email.com HTTP/1.1\r
Host: www.example.com"
    );
}
