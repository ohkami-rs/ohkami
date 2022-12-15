use std::{str::Lines, ops::RangeInclusive};
use crate::{
    components::{method::Method, json::JSON},
    response::Response,
    result::{Result, ElseResponse},
    utils::map::{RangeMap, RANGE_MAP_SIZE},
};

#[cfg(feature = "sqlx")]
use async_std::sync::Arc;
#[cfg(feature = "postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;


pub fn parse_request_lines(
    // lines: &'l mut Lines
    mut lines: Lines
) -> Result<(
    Method,
    &str/*path*/,
    Option<RangeInclusive<usize>>/*path param*/,
    Option<RangeMap>/*query param*/,
    // headers,
    Option<JSON>/*request body*/,
)> {
    let line = lines.next()
        ._else(|| Response::BadRequest("empty request"))?;
    (!line.is_empty())
        ._else(|| Response::BadRequest("can't find request status line"))?;

    let (method, path_str) = line
        .strip_suffix(" HTTP/1.1")
        ._else(|| Response::NotImplemented("I can't handle protocols other than `HTTP/1.1`"))?
        .split_once(' ')
        ._else(|| Response::BadRequest("invalid request line format"))?;

    let (path, query) = extract_query(path_str, method.len() + 1/*' '*/)?;
    let /*(path, param)*/ param = extract_param(path, method.len() + 1/*' '*/);

    while let Some(line) = lines.next() {
        /*
            TODO: header parsing
        */
        if line.is_empty() {break}
    }

    let body = lines.next().map(|line| JSON::from_str(line));

    Ok((
        Method::parse(method)?,
        path.trim_end_matches('/'),
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
    (queries.count() <= RANGE_MAP_SIZE)
        ._else(|| Response::BadRequest("Sorry, I can't handle more than 4 query params"))?;
    
    let mut map = RangeMap::new();
    let (mut index, mut read_pos) = (0, offset + path_part.len() + 1/*'?'*/);
    for (key, value) in queries {
        let (key, value) = (
            (read_pos+1)..=(read_pos+key.len()),
            (read_pos+key.len()+1/*'='*/ +1)..=(read_pos+key.len()+1/*'='*/ +value.len()),
        );
        map.insert(index, key, value)
    }

    Ok((path_part, Some(map)))
}
fn extract_param(
    path:   &str,
    offset: usize
) -> Option<RangeInclusive<usize>> {
    if path.ends_with('/') {return None}
    Some(
            offset+1 + path.rfind('/')? + 1
        ..= offset+1 + path.len()
    )
}


// pub(crate) fn read_request_body(lines: &mut Lines) -> Option<JSON> {
//     // ==========================
//     // TODO: performe this in header parsing
//     while let Some(line) = lines.next() {
//         if line.is_empty() {break}
//     }
//     // ==========================
// 
//     lines.next().map(|body| JSON::from_str(body))
// }