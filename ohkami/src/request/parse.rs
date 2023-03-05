use std::str::Lines;
use super::{QueryParams, Headers, REQUEST_BUFFER_SIZE, Request};

impl<'buf> Request<'buf> {
    #[inline] pub(crate) fn parse(buffer: &'buf [u8; REQUEST_BUFFER_SIZE]) -> Self {
        let mut lines = unsafe {std::str::from_utf8_unchecked(buffer.trim_ascii_end())}.lines();

        let (method, path, query_params) = method_path_query(&mut lines);
        let (headers, body) = headers_body(&mut lines);

        Self { method, path, query_params, headers, body }
    }
}

#[inline] fn method_path_query<'buf>(lines: &mut Lines<'buf>) -> (/*method*/&'buf str, /*path*/&'buf str, QueryParams<'buf>) {
    let (method_path, _) = lines.next().unwrap().rsplit_once(' ').unwrap();
    let (method, path) = method_path.split_once(' ').unwrap();
    let (path, query) = extract_query(path);
    (method, path, query)
}
#[inline] fn extract_query<'buf>(path: &'buf str) -> (/*path*/&'buf str, QueryParams) {
    let mut queries = QueryParams::new();
    if let Some((path, query_str)) = path.split_once('?') {
        for query in query_str.split('&') {
            let Some((key, value)) = query.split_once('=')
                else {tracing::warn!("invalid query parameter: `{query}`"); continue};
            queries.push(key, value)
        }
        (path, queries)
    } else {
        (path, queries)
    }
}
#[inline] fn headers_body<'buf>(lines: &mut Lines<'buf>) -> (Headers<'buf>, Option<&'buf str>) {
    let mut headers = Headers::new();

    while let Some(next_line) = lines.next() {
        if next_line.is_empty() {break}

        let Some((key, value)) = next_line.split_once(':')
            else {tracing::warn!("invalid request header: `{next_line}`"); continue};
        headers.append(key, &value[1..])
    }

    (headers, lines.next())
}
