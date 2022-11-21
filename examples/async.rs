use std::{collections::HashMap, pin::Pin, task::Poll};

use async_http::Request;
use futures::Future;


fn main() {
    let m = Map::new();
    // let s = sample2;

    m.resister(sample2);
}

trait Handler {}

struct Map(
    usize,
    HashMap<
        usize,
        fn(Request) -> Res
    >
);impl Map {
    fn new() -> Self {
        Self(0, HashMap::new())
    }
    fn resister(&mut self, f: fn(Request) -> Res) {
        self.0 += 1;
        self.1.insert(self.0, f);
    }
}

struct Res(usize);
impl Future for Res {
    type Output = usize;
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.0)
    }
}

async fn sample1<'r>(_: Request<'r>) -> Pin<Box<i32>> { Box::pin(100) }
async fn sample2<'r>(_: Request<'r>) -> Res { Res(200) }