use std::{pin::Pin, future::Future};
use crate::{__feature__, Context, request::PathParams};


pub struct Handler(
    Box<dyn
        Fn(__feature__::TcpStream, Context, PathParams) -> Pin<
            Box<dyn
                Future<Output = ()>
                + Send + 'static
            >
        > + Send + Sync + 'static
    >
);
