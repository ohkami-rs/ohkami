use std::{sync::Arc, collections::HashSet};

use crate::{
    components::{consts::BUF_SIZE, method::Method, json::JSON},
    response::Response,
    context::Context, result::Result,
};

pub(crate) fn parse_stream<'buf>(
    buffer: &'buf [u8; BUF_SIZE],
    allow_origin: Arc<HashSet<&'static str>>,
) -> Result<(
    Method,
    &'buf str,
    Context<'buf>
)> {
    let mut lines = std::str::from_utf8(buffer)?
        .trim_end()
        .lines();

    let request_line = lines.next().ok_or_else(|| Response::BadRequest("empty request"))?;
    let (method, path) = parse_request_line(request_line)?;

    while let Some(line) = lines.next() {
        if line.is_empty() {break}

        if line.starts_with("Origin: ") {
            let origin = &line[8..];
            if !allow_origin.contains(origin) {
                return Err(Response::Forbidden("that origin is not allowed"))
            }
            break  // 今のところ Origin 以外見ないので
        }

        // TODO: handle BasicAuth
    }

    let request_context = Context {
        pool:  None,
        param: None,
        body:
            if let Some(request_body) = lines.next() {
                Some(JSON::from_str_unchecked(request_body))
            } else {None}
    };

    Ok((method, path, request_context))
}

fn parse_request_line(line: &str) -> Result<(Method, &str)> {
    if line.is_empty() {
        return Err(Response::BadRequest("can't find request status line"))
    }
    let (method, path) = line
        .strip_suffix(" HTTP/1.1")
        .ok_or_else(|| Response::NotImplemented("I can't handle protocols other than `HTTP/1.1`"))?
        .split_once(' ')
        .ok_or_else(|| Response::BadRequest("invalid request line format"))?;
    Ok((Method::parse(method)?, path))
}