use std::future::IntoFuture;
use worker::kv::{KvStore, KvError, ToRawKvValue, GetOptionsBuilder};
use worker::wasm_bindgen_futures::JsFuture;
use worker::wasm_bindgen::JsValue;


pub struct KV(pub(super) KvStore);

pub struct Error(KvError);

// SAFETY: This is in `rt_worker`
const _: () = {
    unsafe impl Send for KV {}
    unsafe impl Sync for KV {}

    unsafe impl Send for Error {}
    unsafe impl Sync for Error {}
};

const _: () = {
    struct GetValue(GetOptionsBuilder);
    unsafe impl Send for GetValue {}
    unsafe impl Sync for GetValue {}

    impl KV {
        pub fn get(&self, key: &str) -> GetValue {
            GetValue(self.0.get(key))
        }
        pub fn get_with_cache_ttl(&self, key: &str, ttl: u64) -> GetValue {
            GetValue(self.0.get(key).cache_ttl(ttl))
        }
    }

    // impl GetValue {
    //     pub async fn 
    // }

    struct 
    impl IntoFuture for GetValue {
        type IntoFuture = JsFuture;
        type Output = Result<String, Error>;
        fn into_future(self) -> Self::IntoFuture {
            self.0.
        }
    }
};
