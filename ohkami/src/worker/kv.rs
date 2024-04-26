use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use serde::Deserialize;
use worker::kv::{GetOptionsBuilder, KvError, KvStore, ListOptionsBuilder, ListResponse, PutOptionsBuilder, ToRawKvValue};
use worker::wasm_bindgen::JsValue;
use super::{SendFuture, Error};


pub struct KV(pub(super) KvStore);
unsafe impl Send for KV {}
unsafe impl Sync for KV {}

const _: (/* get text */) = {
    impl KV {
        pub fn get(&self, key: &str) -> Get {
            Get(self.0.get(key))
        }
    }

    pub struct Get(GetOptionsBuilder);
    unsafe impl Send for Get {}
    unsafe impl Sync for Get {}

    impl Get {
        pub fn cache_ttl(self, ttl: u64) -> Self {
            Self(self.0.cache_ttl(ttl))
        }
    }

    impl IntoFuture for Get {
        type Output     = Result<String, Error>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        fn into_future(self) -> Self::IntoFuture {
            // SendFuture(async {match self.0.text().await {
            //     Ok(option) => option.ok_or_else(|| Error(KvError::JavaScript(JsValue::from_str("Specified  `{\"type\": \"text\"}` but not an text")))),
            //     Err(error) => Err(Error(error)),
            // }})
            SendFuture(async {
                self.0.text().await
                    .map_err(Error::KV)?
                    .ok_or_else(|| Error::KV(KvError::JavaScript(JsValue::from_str("Specified  `{\"type\": \"text\"}` but not an text"))))
            })
        }
    }
};

const _: (/* get json */) = {
    impl KV {
        pub fn get_<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Get<T> {
            Get(self.0.get(key), PhantomData)
        }
    }

    pub struct Get<T: for<'de> Deserialize<'de>>(
        GetOptionsBuilder,
        PhantomData<fn()->T>
    );

    unsafe impl<T: for<'de> Deserialize<'de>> Send for Get<T> {}
    unsafe impl<T: for<'de> Deserialize<'de>> Sync for Get<T> {}

    impl<T: for<'de> Deserialize<'de>> Get<T> {
        pub fn cache_ttl(self, ttl: u64) -> Self {
            Self(self.0.cache_ttl(ttl), PhantomData)
        }
    }

    impl<T: for<'de> Deserialize<'de>> IntoFuture for Get<T> {
        type Output     = Result<T, Error>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        fn into_future(self) -> Self::IntoFuture {
            SendFuture(async {
                self.0.json().await
                    .map_err(Error::KV)?
                    .ok_or_else(|| Error::KV(KvError::JavaScript(JsValue::from_str("Specified `{\"type\": \"json\"}` but got `null`"))))
            })
        }
    }
};

const _: (/* put */) = {
    impl KV {
        pub fn put(&self, key: &str, value: impl ToRawKvValue) -> Put {
            Put(self.0.put(key, value).map_err(Error::KV))
        }
    }

    pub struct Put(Result<PutOptionsBuilder, Error>);
    unsafe impl Send for Put {}
    unsafe impl Sync for Put {}

    impl Put {
        pub fn expiration(self, timestamp: u64) -> Self {
            Self(self.0.map(|put| put.expiration(timestamp)))
        }

        pub fn expiration_ttl(self, ttl: u64) -> Self {
            Self(self.0.map(|put| put.expiration_ttl(ttl)))
        }

        pub fn metadata(self, metadata: impl serde::Serialize) -> Self {
            Self(match self.0 {
                Ok(put) => put.metadata(metadata).map_err(Error::KV),
                Err(e)  => Err(e),
            })
        }
    }

    impl IntoFuture for Put {
        type Output     = Result<(), Error>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        fn into_future(self) -> Self::IntoFuture {
            SendFuture(async {
                self.0?.execute().await
                    .map_err(Error::KV)
            })
        }
    }
};

const _: (/* list */) = {
    impl KV {
        pub fn list(&self) -> List {
            List(self.0.list())
        }
    }

    pub struct List(ListOptionsBuilder);
    unsafe impl Send for List {}
    unsafe impl Sync for List {}

    impl List {
        pub fn cursor(self, cursor: impl Into<String>) -> Self {
            Self(self.0.cursor(cursor.into()))
        }

        pub fn prefix(self, prefix: impl Into<String>) -> Self {
            Self(self.0.prefix(prefix.into()))
        }

        pub fn limit(self, limit: u64) -> Self {
            Self(self.0.limit(limit))
        }
    }

    impl IntoFuture for List {
        type Output     = Result<ListResponse, Error>;
        type IntoFuture = impl Future<Output = Self::Output> + Send;

        fn into_future(self) -> Self::IntoFuture {
            SendFuture(async {
                self.0.execute().await
                    .map_err(Error::KV)
            })
        }
    }
};

const _: (/* delete */) = {
    impl KV {
        pub async fn delete(&self, key: &str) -> Result<(), Error> {
            self.0.delete(key).await.map_err(Error::KV)
        }
    }
};
