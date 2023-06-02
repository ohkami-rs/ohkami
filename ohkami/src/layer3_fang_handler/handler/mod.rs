use std::{pin::Pin, future::Future};
use crate::{__dep__, Context, request::PathParams};


pub struct Handler(
    Box<dyn
        Fn(__dep__::TcpStream, Context, PathParams) -> Pin<
            Box<dyn
                Future<Output = ()>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
);
