use std::collections::HashMap;
use async_std::{
    sync::Arc,
    io::{ReadExt, WriteExt},
    net::{TcpStream, TcpListener},
    stream::StreamExt, task,
};
use futures::channel::mpsc;

use crate::{
    context::Context,
    components::{consts::BUF_SIZE, method::Method},
    request::Request,
    response::Response,
    utils::parse::parse_stream
};


pub struct Server(
    HashMap<
        (Method, &'static str, bool),
        fn(Request) -> Context<Response>,
    >
);
impl Server {
    async fn serve_on(self, tcp_address: String) -> Context<()> {
        let listener = TcpListener::bind(tcp_address).await?;
        let mut incoming = listener.incoming();

        // let (receiver, sender) = mpsc::unbounded();
        let handler_map = Arc::new(self.0);

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let handle = task::spawn(
                handle_stream(stream, Arc::clone(&handler_map))
            );
        }

        Ok(())
    }
}

async fn handle_stream(
    mut stream: TcpStream,
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        fn(Request) -> Context<Response>,
    >>,
) -> Context<()> {
    let mut buffer = [b' '; BUF_SIZE];
    stream.read(&mut buffer).await?;

    let (method, path_str, request) = parse_stream(&buffer)?;
    match handle_request(handler_map, method, path_str, request) {
        Ok(res)  => res,
        Err(res) => res,
    }.write_to_stream(
        &mut stream
    ).await?;

    stream.flush().await?;
    Ok(())
}
fn handle_request<'path>(
    handler_map: Arc<HashMap<
        (Method, &'static str, bool),
        fn(Request) -> Context<Response>,
    >>,
    method:      Method,
    path_str:    &'path str,
    mut request: Request<'path>,
) -> Context<Response> {
    let handler = 
        if let Some(handler) = handler_map.get(&(method, path_str, false)) {
            handler
        } else {
            let (path, param) = path_str.rsplit_once('/')
                .ok_or_else(|| Response::BadRequest(format!(
                    "invalid request path format: {path_str}"
                )))?;
            let handler = handler_map.get(&(method, path, true))
                .ok_or_else(||
                    if let Some(_) = handler_map.get(&(method, path_str, true)) {
                        Response::BadRequest(format!(
                            "expected a path parameter"
                        ))
                    } else {
                        Response::NotFound(format!(
                            "handler for `{method} {path_str}` is not found"
                        ))
                    }
                )?;
            request.param = Some(param);
            handler
        };
    handler(request)
}


/*
#![allow(unused)]
fn main() {
extern crate async_std;
use async_std::{
    net::{TcpListener, ToSocketAddrs},
    prelude::*,
    task,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use async_std::{
    io::BufReader,
    net::TcpStream,
};

async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        println!("Accepting from: {}", stream.peer_addr()?);
        let _handle = task::spawn(connection_loop(stream)); // 1
    }
    Ok(())
}

async fn connection_loop(stream: TcpStream) -> Result<()> {
    let reader = BufReader::new(&stream); // 2
    let mut lines = reader.lines();

    let name = match lines.next().await { // 3
        None => Err("peer disconnected immediately")?,
        Some(line) => line?,
    };
    println!("name = {}", name);

    while let Some(line) = lines.next().await { // 4
        let line = line?;
        let (dest, msg) = match line.find(':') { // 5
            None => continue,
            Some(idx) => (&line[..idx], line[idx + 1 ..].trim()),
        };
        let dest: Vec<String> = dest.split(',').map(|name| name.trim().to_string()).collect();
        let msg: String = msg.to_string();
    }
    Ok(())
}
}

*/