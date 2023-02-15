use std::str::Lines;
use crate::{
    response::Response,
    result::{Result, ElseResponse},
    components::{method::Method, },
    utils::{buffer::BufRange, range::{RangeMap, RANGE_MAP_SIZE}},
};


pub(crate) fn parse_request(mut lines: Lines) -> Result<(
    Method,
    String/*path*/,
    Option<RangeMap>/*query param*/,
    HeaderRangeMap,
    Option<String>/*request body*/,
)> {
    let line = lines.next()
        ._else(|| Response::BadRequest("empty request"))?;

    let (method_str, path_str) = line
        .strip_suffix(" HTTP/1.1")
        ._else(|| Response::NotImplemented("I can't handle protocols other than `HTTP/1.1`"))?
        .split_once(' ')
        ._else(|| Response::BadRequest("invalid request line format"))?;

    tracing::info!("request: {} {}", method_str, path_str);

    let (path, query) = extract_query(path_str, method_str.len() - 1/*' '*/)?;

    let mut header_map = HeaderRangeMap::new();
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
