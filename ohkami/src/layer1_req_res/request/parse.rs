use super::{Request, QUERIES_LIMIT, HEADERS_LIMIT};
use crate::{
    Error,
    layer0_lib::{BUFFER_SIZE, Buffer, Method, List},
};


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
            if buffer[header_start] == b'\r' {break}

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
                    b'\r' => break,
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
                0 => break,
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
