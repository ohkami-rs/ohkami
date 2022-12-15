use std::str::Lines;
use crate::{
    components::{method::Method, json::JSON},
    response::Response,
    context::Context,
    result::{Result, ElseResponse},
    utils::map::RangeMap,
};

#[cfg(feature = "sqlx")]
use async_std::sync::Arc;
#[cfg(feature = "postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;

use super::map::RANGE_MAP_SIZE;



// pub(crate) fn parse_stream<'buf>(
//     buffer: &'buf [u8; BUF_SIZE],
//     #[cfg(feature = "sqlx")]
//     connection_pool: Arc<ConnectionPool>,
// ) -> Result<(
//     Method,
//     &'buf str,
//     Context
// )> {
//     let mut lines = std::str::from_utf8(buffer)?
//         .trim_end()
//         .lines();
// 
//     let request_line = lines.next()
//         ._else(|| Response::BadRequest("empty request"))?;
//     
//     tracing::debug!("got a request: {}", request_line);
//     let (
//         method,
//         path,
//         param,
//         query
//     ) = parse_request_line(request_line)?;
// 
//     while let Some(line) = lines.next() {
//         if line.is_empty() {break}
// 
//         // TODO: handle BasicAuth
//     }
// 
//     let request_context = Context {
//         param,
//         query,
//         body: lines.next().map(|request_body| JSON::from_str(request_body)),
// 
//         #[cfg(feature = "sqlx")]
//         pool:  connection_pool,
//     };
// 
//     Ok((method, path, request_context))
// }

pub fn parse_request_line<'l>(
    lines: &'l mut Lines
) -> Result<(Method, &'l str, Option<RangeMap>, Option<RangeMap>)> {
    let line = lines.next()
        ._else(|| Response::BadRequest("empty request"))?;
    (!line.is_empty())
        ._else(|| Response::BadRequest("can't find request status line"))?;

    let (method, path_str) = line
        .strip_suffix(" HTTP/1.1")
        ._else(|| Response::NotImplemented("I can't handle protocols other than `HTTP/1.1`"))?
        .split_once(' ')
        ._else(|| Response::BadRequest("invalid request line format"))?;

    let (path_part, query) = extract_query(path_str, method.len() + 1/*' '*/)?;
    let (path, param) = extract_param(path_part, method.len() + 1/*' '*/)?;

    Ok((Method::parse(method)?, path.trim_end_matches('/'), param, query))
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
            (read_pos+1)..(read_pos+key.len()),
            (read_pos+key.len()+1/*'='*/ +1)..(read_pos+key.len()+1/*'='*/ +value.len()),
        );
        map.insert(index, key, value)
    }
    Ok((path_part, Some(map)))
}
fn extract_param(
    path_str: &str
) -> Result<(&str, Option<RangeMap>)> {
    let (rest, tail) = path_str.rsplit_once('/').unwrap();
    if let Ok(param) = tail.parse::<u32>() {
        Ok((rest, Some(param)))
    } else {
        Ok((path_str, None))
    }
}