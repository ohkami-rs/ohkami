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


pub enum Error {
    Worker(worker::Error),
    KV(worker::kv::KvError),
}
const _: () = {
    unsafe impl Send for Error {}
    unsafe impl Sync for Error {}

    impl std::fmt::Debug for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::KV(e)     => e.fmt(f),
                Self::Worker(e) => e.fmt(f),
            }
        }
    }
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::KV(e)     => e.fmt(f),
                Self::Worker(e) => e.fmt(f),
            }
        }
    }
    impl std::error::Error for Error {}
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

    pub fn D1(&self, name: &'static str) -> Result<d1::D1, worker::Error> {
        self.0.d1(name).map(d1::D1)
    }
}


#[cfg(test)]
async fn __usavility__(bindings: Bindings<'_>) {
    #[derive(serde::Deserialize)]
    #[allow(unused)]
    struct Point {
        x: usize,
        y: usize,
    }

    #[derive(serde::Deserialize)]
    #[allow(unused)]
    struct User {
        name:     String,
        age:      u8,
        favorite: String,
    }

    let kv = bindings.KV("MY_KV").unwrap();
    {
        let _: String = kv.get("text").await.unwrap();
        let _: String = kv.get("text").cache_ttl(1024).await.unwrap();
        let _: Point  = kv.get_::<Point>("point").await.unwrap();
        let _: Point  = kv.get_::<Point>("point").cache_ttl(1024).await.unwrap();
        kv.delete("key").await.unwrap();
        let _: worker::kv::ListResponse = kv.list().await.unwrap();
        let _: worker::kv::ListResponse = kv.list().cursor("c").await.unwrap();
        let _: worker::kv::ListResponse = kv.list().limit(42).await.unwrap();
        let _: worker::kv::ListResponse = kv.list().prefix("p").await.unwrap();
        kv.put("k", "v").await.unwrap();
        kv.put("k", "v").expiration(1234567890).await.unwrap();
        kv.put("k", "v").expiration_ttl(543210).await.unwrap();
    }

    let d1 = bindings.D1("MY_D1").unwrap();
    {
        let _ = d1.query("SELECT * FROM users WHERE name = ?1, age > ?2, favorite = ?3")
            .bind(("Joe", 42, "Christmas"))
            .all::<User>().await.unwrap();

        d1.query("DELETE * FROM users LIMIT 1024").await.unwrap();
    }
}
