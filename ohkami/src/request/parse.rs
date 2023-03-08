use std::str::Lines;
use super::{QueryParams, Headers, REQUEST_BUFFER_SIZE, Request, Method, BufRange};

impl Request {
    #[inline] pub(crate) fn parse(buffer: [u8; REQUEST_BUFFER_SIZE]) -> Request {
        let mut start = 0;

        let method = {
            let mut end = start;
            for b in &buffer[start..] {
                if b == b' ' {break}
                end += 1;
            }
            let method = Method::parse(&buffer[start..end]);
            start = end + 1;
            method
        };

        let mut queries = QueryParams::new();
        let path = {
            let mut queries_start = start;
            let mut end = start;
            for b in &buffer[start..] {
                end += 1;
                match b {
                    b'?' => {queries_start = end + 1; break},
                    b' ' => break,
                    _ => (),
                }
            }
            let path = BufRange {start, end};

            if queries_start > start {
                start = queries_start;

                loop {
                    let mut is_final = false;

                    let key = {
                        let mut end = start;
                        for b in &buffer[start..] {
                            end += 1;
                            if b == b'=' {break}
                        }
                        BufRange {start, end}
                    };
                    let value = {
                        let mut end = start;
                        for b in &buffer[start..] {
                            end += 1;
                            match b {
                                b' ' => {is_final = true; break},
                                b'&' => {break},
                                _ => (),
                            }
                        }
                        BufRange {start, end}
                    };

                    queries.push(key, value);
                    if is_final {break}
                }
            }
            
            start = end + 1/* ' ' */;
            path
        };

        let _/* HTTP version */ = {
            let mut end = start;
            for b in &buffer[start..] {
                if b == b'\r' {break}
                end += 1 ;
            }
            start = end + 1 + 1/* '\n' */;
        };

        let mut headers = Headers::new();
        let mut body = None;
        loop {
            if start >= REQUEST_BUFFER_SIZE
            || buffer[start] == b' ' {
                break
            } else if buffer[start] == b'\r' {
                start += 1/* '\n' */ + 1;

                let mut end = start;
                for b in &buffer[start..] {
                    if b == b'\r' {break}
                    end += 1
                }
                body.replace(BufRange {start, end})
            }

            let key = {
                let mut end = start;
                for b in &buffer[start..] {
                    if b == b':' {break}
                    end += 1
                }
                let key = BufRange {start, end};
                start = end + 1 + 1/* ' ' */;
                key
            };

            let value = {
                let mut end = start;
                for b in &buffer[start..] {
                    if b == b'\r' {break}
                    end += 1
                }
                let value = BufRange {start, end};
                start = end + 1 + 1/* '\n' */;
                value
            };

            headers.append(key, value)
        }

        Self { buffer, method, path, queries, headers, body }
    }
}

// #[inline] fn method_path_query<'buf>(lines: &'buf mut Lines<'buf>) -> (/*method*/&'buf str, /*path*/&'buf str, QueryParams<'buf>) {
//     let (method_path, _) = lines.next().unwrap().rsplit_once(' ').unwrap();
//     let (method, path) = method_path.split_once(' ').unwrap();
//     let (path, query) = extract_query(path);
//     (method, path, query)
// }
// #[inline] fn extract_query<'buf>(path: &'buf str) -> (/*path*/&'buf str, QueryParams) {
//     let mut queries = QueryParams::new();
//     if let Some((path, query_str)) = path.split_once('?') {
//         for query in query_str.split('&') {
//             let Some((key, value)) = query.split_once('=')
//                 else {tracing::warn!("invalid query parameter: `{query}`"); continue};
//             queries.push(key, value)
//         }
//         (path, queries)
//     } else {
//         (path, queries)
//     }
// }
// #[inline] fn headers_body<'buf>(lines: &'buf mut Lines<'buf>) -> (Headers<'buf>, Option<&'buf str>) {
//     let mut headers = Headers::new();
// 
//     while let Some(next_line) = lines.next() {
//         if next_line.is_empty() {break}
// 
//         let Some((key, value)) = next_line.split_once(':')
//             else {tracing::warn!("invalid request header: `{next_line}`"); continue};
//         headers.append(key, &value[1..])
//     }
// 
//     (headers, lines.next())
// }
// 