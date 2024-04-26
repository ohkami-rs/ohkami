pub mod d1;
pub mod kv;

use std::future::Future;
use crate::FromRequest;


// #[allow(non_snake_case)]
// fn AssertSend<T: Send>() {}

struct SendFuture<F: Future>(F);
const _: () = {
    unsafe impl<F: Future> Send for SendFuture<F> {}
    unsafe impl<F: Future> Sync for SendFuture<F> {}
    impl<F: Future> Future for SendFuture<F> {
        type Output = F::Output;

        fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            unsafe {self.map_unchecked_mut(|this| &mut this.0)}.poll(cx)
        }
    }
};


pub struct Bindings<'worker>(&'worker worker::Env);

impl<'req> FromRequest<'req> for Bindings<'req> {
    type Error = std::convert::Infallible;
    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        Some(Ok(Self(req.env())))
    }
}

#[allow(non_snake_case)]
impl<'worker> Bindings<'worker> {
    pub fn KV(&self, name: &'static str) -> Result<kv::KV, worker::Error> {
        self.0.kv(name).map(kv::KV)
    }
}
