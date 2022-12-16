use std::str::Lines;
use crate::{
    components::{method::Method, json::JSON},
    response::Response,
    result::{Result, ElseResponse},
    utils::{buffer::BufRange, map::{RangeMap, RANGE_MAP_SIZE}},
};


pub(crate) fn parse_request_lines(
    // lines: &'l mut Lines
    mut lines: Lines
) -> Result<(
    Method,
    String/*path*/,
    Option<BufRange>/*path param*/,
    Option<RangeMap>/*query param*/,
    // headers,
    Option<JSON>/*request body*/,
)> {
    let line = lines.next()
        ._else(|| Response::BadRequest("empty request"))?;
    (!line.is_empty())
        ._else(|| Response::BadRequest("can't find request status line"))?;

    let (method_str, path_str) = line
        .strip_suffix(" HTTP/1.1")
        ._else(|| Response::NotImplemented("I can't handle protocols other than `HTTP/1.1`"))?
        .split_once(' ')
        ._else(|| Response::BadRequest("invalid request line format"))?;

    tracing::info!("got a request: {} {}", method_str, path_str);

    let (path, query) = extract_query(path_str, method_str.len() - 1/*' '*/)?;
    let /*(path, param)*/ param = extract_param(path, method_str.len() - 1/*' '*/);

    while let Some(line) = lines.next() {
        /*
            TODO: header parsing
        */
        if line.is_empty() {break}
    }

    let body = lines.next().map(|line| JSON::from_str(line));

    Ok((
        Method::parse(method_str)?,
        (if path=="/" {path} else {path.trim_end_matches('/')}).to_owned(),
        param,
        query,
        body
    ))
}
fn extract_query(
    path_str: &str,
    offset:   usize,
) -> Result<(&str, Option<RangeMap>)> {
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
            Response::BadRequest("Sorry, I can't handle more than 4 query params")
        )?;
        map.insert(i,
            BufRange::new(read_pos+1, read_pos+key.len()),
            BufRange::new(read_pos+key.len()+1/*'='*/ +1, read_pos+key.len()+1/*'='*/ +value.len()),
        );
        read_pos += key.len()+1/*'='*/ +value.len() + 1
    }

    Ok((path_part, Some(map)))
}
fn extract_param(
    path:   &str,
    offset: usize
) -> Option<BufRange> {
    if path.ends_with('/') {return None}
    Some(BufRange::new(
        offset+1 + path.rfind('/')?+1 + 1,
        offset+1 + path.len()
    ))
}
