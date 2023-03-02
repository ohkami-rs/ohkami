mod method; use method::Method;
use std::{ops::Range, str::Lines};
use super::{QueryParams, Headers, REQUEST_BUFFER_SIZE};


#[inline] pub(super) fn parse_request<'buf>(
    buffer: [u8; REQUEST_BUFFER_SIZE]
) -> crate::Result<(
    Method,
    /*path*/&'buf str,
    QueryParams,
    Headers,
    Option<Range<usize>>,
)> {
    let mut lines = unsafe {std::str::from_utf8_unchecked(buffer.trim_ascii_end())}.lines();

    let (method, path, query) = method_path_query(&mut lines);
    let (headers, body) = headers_body(&mut lines);

    todo!()
}

#[inline] fn method_path_query<'buf>(lines: &mut Lines<'buf>) -> (Method, /*path*/&'buf str, QueryParams) {
    let (method_path, _) = lines.next().unwrap().rsplit_once(' ').unwrap();
    let (method, path) = method_path.split_once(' ').unwrap();
    let (path, query) = extract_query(path);
    (Method::parse(method), path, query)
}
#[inline] fn extract_query<'buf>(path: &'buf str) -> (/*path*/&'buf str, QueryParams) {
    let mut query = QueryParams::new();
    if let Some((path, query_str)) = path.split_once('&') {
        (path, QueryParams::/* TODO */)
    } else {
        (path, query)
    }
}
#[inline] fn headers_body(lines: &mut Lines) -> (Headers, Option<Range<usize>>) {
    let mut headers = Headers::new();

    loop {
        let Some(next_line) = lines.next() else {
            todo!(/* return Headers imediately */)
        }; if next_line.is_empty() {
            if let Some(body) = lines.next() {
                return (headers, /* ... */)
            } else {
                return (headers, None)
            }
        }
    }
}

/*
pub(crate) fn parse_request<'buf>(mut lines: Lines<'buf>) -> Result<RawRequest<'buf>, ()> {
    let line = lines.next()
        ._else(|| Response::BadRequest("empty request"))?;

    let (method_str, path_str) = line
        .strip_suffix(" HTTP/1.1")
        ._else(|| Response::NotImplemented("I can't handle protocols other than `HTTP/1.1`"))?
        .split_once(' ')
        ._else(|| Response::BadRequest("invalid request line format"))?;

    tracing::info!("request: {} {}", method_str, path_str);

    let (path, query) = extract_query(path_str, method_str.len() - 1/*' '*/)?;

    let mut header_map = RequestHeaders::new();
    let mut offset = line.len() + 2/*'\r\n'*/;
    let mut is_json = false;
    while let Some(line) = lines.next() {
        if line.is_empty() {break}

        let colon = line.find(':').unwrap();
        header_map.push(
            BufRange::new(offset, offset+colon-1),
            BufRange::new(offset+colon+1/*' '*/+1, offset+line.len()-1)
        );

        if !is_json
        && &line[..colon]=="Content-Type"
        && &line[colon+2..colon+2+16]=="application/json" {
            is_json = true
        }

        offset += line.len() + 2/*'\r\n'*/
    }

    let body = if is_json {
        Some(
            lines.next()
                ._else(|| Response::BadRequest("Headers has `Content-Type: application/json` but no request body was found"))?
                .to_owned()
        )
    } else {
        None
    };

    Ok((
        Method::parse(method_str)?,
        path.trim_end_matches('/').to_owned(),
        query,
        header_map,
        body
    ))
}
fn extract_query(
    path_str: &str,
    offset:   usize,
) -> std::result::Result<(&str, Option<RangeMap>)> {
    let Some((path_part, query_part)) = path_str.split_once('?')
        else {return Ok((path_str, None))};
    
    let queries = query_part.split('&')
        .map(|key_value| key_value
            .split_once('=')
            .expect("invalid query parameter format")
        );
    
    let mut map = RangeMap::new();
    let mut read_pos = offset + path_part.len() + 1/*'?'*/ + 1;
    for (i, (key, value)) in queries.enumerate() {
        (i < RANGE_MAP_SIZE)._else(||
            Response::BadRequest(format!("Sorry, ohkami doesn't handle more than {} query params", RANGE_MAP_SIZE))
        )?;
        map.insert(i,
            BufRange::new(read_pos+1, read_pos+key.len()),
            BufRange::new(read_pos+key.len()+1/*'='*/ +1, read_pos+key.len()+1/*'='*/ +value.len()),
        );
        read_pos += key.len()+1/*'='*/ +value.len() + 1
    }

    Ok((path_part, Some(map)))
}

*/