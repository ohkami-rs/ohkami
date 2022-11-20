use std::collections::HashMap;

use async_std::{
    io::{Read, ReadExt},
    net::{TcpStream, TcpListener},
    stream::StreamExt, task,
};

use crate::{context::Context, components::consts::BUF_SIZE};

pub struct Server<'method, 'path>(
    HashMap<
        (&'method Method, &'path str, bool),
        fn(Request) -> Context<Response>,
    >
);
impl<'method, 'path> Server<'method, 'path> {
    async fn serve_on(&self, tcp_address: String) -> Context<()> {
        let listener = TcpListener::bind(tcp_address).await?;
        let mut incoming = listener.incoming();

        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            let handle = task::spawn(self.handle_request(stream));
        }

        Ok(())
    }
    async fn handle_request(&self, stream: TcpStream) -> Context<()> {
        let buffer = [b' '; BUF_SIZE];
        stream.read(&mut buffer).await?;

        
    }
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