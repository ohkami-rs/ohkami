#[cfg(feature = "sqlx")]
use async_std::sync::Arc;

use crate::{
    components::{consts::BUF_SIZE, method::Method, json::JSON},
    response::Response,
    context::Context, result::{Result, ElseResponse},
};

#[cfg(feature = "postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;

use super::hash::StringHashMap;


pub(crate) fn parse_stream<'buf>(
    buffer: &'buf [u8; BUF_SIZE],
    #[cfg(feature = "sqlx")]
    connection_pool: Arc<ConnectionPool>,
) -> Result<(
    Method,
    &'buf str,
    Context
)> {
    let mut lines = std::str::from_utf8(buffer)?
        .trim_end()
        .lines();

    let request_line = lines.next()
        ._else(|| Response::BadRequest("empty request"))?;
    
    tracing::debug!("got a request: {}", request_line);
    let (
        method,
        path,
        param,
        query
    ) = parse_request_line(request_line)?;

    while let Some(line) = lines.next() {
        if line.is_empty() {break}

        // TODO: handle BasicAuth
    }

    let request_context = Context {
        param,
        query,
        body: lines.next().map(|request_body| JSON::from_str(request_body)),

        #[cfg(feature = "sqlx")]
        pool:  connection_pool,
    };

    Ok((method, path, request_context))
}

fn parse_request_line(
    line: &str
) -> Result<(Method, &str, Option<u32>, Option<StringHashMap>)> {
    (!line.is_empty())
        ._else(|| Response::BadRequest("can't find request status line"))?;

    let (method, path_str) = line
        .strip_suffix(" HTTP/1.1")
        ._else(|| Response::NotImplemented("I can't handle protocols other than `HTTP/1.1`"))?
        .split_once(' ')
        ._else(|| Response::BadRequest("invalid request line format"))?;

    let (path_part, query) = extract_query(path_str)?;
    let (path, param) = extract_param(path_part)?;

    Ok((Method::parse(method)?, path.trim_end_matches('/'), param, query))
}

fn extract_query(
    path_str: &str
) -> Result<(&str, Option<StringHashMap>)> {
    let Some((path_part, query_part)) = path_str.split_once('?')
        else {return Ok((path_str, None))};

    let mut map = StringHashMap::new()?;
    query_part.split('&')
        .map(|key_value| key_value
            .split_once('=')
            .expect("invalid query parameter format")
        )
        .for_each(|(key, value)|
            map.insert(key, value.to_owned())
        );
    Ok((path_part, Some(map)))
}

fn extract_param(
    path_str: &str
) -> Result<(&str, Option<u32>)> {
    let (rest, tail) = path_str.rsplit_once('/').unwrap();
    if let Ok(param) = tail.parse::<u32>() {
        Ok((rest, Some(param)))
    } else {
        Ok((path_str, None))
    }
}