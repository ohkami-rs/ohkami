use crate::{
    __feature__,
    layer0_lib::{List, Method, BufRange, Buffer},
};


pub struct Request {
    buffer:  Buffer,
    method:  Method,
    path:    BufRange,
    queries: List<(BufRange, BufRange), 4>,
    headers: List<(BufRange, BufRange), 32>,
    body:    Option<BufRange>,
}

impl Request {
    pub(crate) async fn parse(stream: &mut __feature__::TcpStream) -> Self {
        
    }
}
