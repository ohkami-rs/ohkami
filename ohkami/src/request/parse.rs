use std::str::Lines;
use super::{QueryParams, Headers, REQUEST_BUFFER_SIZE};

#[inline] pub(crate) fn parse_request<'buf>(
    buffer: &'buf [u8; REQUEST_BUFFER_SIZE]
) -> (
    /*method*/&'buf str,
    /*path*/&'buf str,
    QueryParams<'buf>,
    Headers<'buf>,
    Option<&'buf str>,
) {
    let mut lines = unsafe {std::str::from_utf8_unchecked(buffer.trim_ascii_end())}.lines();

    let (method, path, query) = method_path_query(&mut lines);
    let (headers, body) = headers_body(&mut lines);

    todo!()
}

#[inline] fn method_path_query<'buf>(lines: &mut Lines<'buf>) -> (/*method*/&'buf str, /*path*/&'buf str, QueryParams<'buf>) {
    let (method_path, _) = lines.next().unwrap().rsplit_once(' ').unwrap();
    let (method, path) = method_path.split_once(' ').unwrap();
    let (path, query) = extract_query(path);
    (method, path, query)
}
#[inline] fn extract_query<'buf>(path: &'buf str) -> (/*path*/&'buf str, QueryParams) {
    if let Some((path, query_str)) = path.split_once('?') {
        (path, QueryParams::parse(query_str))
    } else {
        (path, QueryParams::new())
    }
}
#[inline] fn headers_body<'buf>(lines: &mut Lines<'buf>) -> (Headers<'buf>, Option<&'buf str>) {
    let mut headers = Headers::new();

    loop {
        let Some(next_line) = lines.next() else {
            todo!(/* return Headers imediately */)
        }; if next_line.is_empty() {
            if let Some(body) = lines.next() {
                return (headers, Some(body))
            } else {
                return (headers, None)
            }
        }
    }
}
