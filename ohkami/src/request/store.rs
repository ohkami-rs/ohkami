use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{Hasher, BuildHasherDefault},
};


pub struct Store(
    Option<Box<
        HashMap<
            TypeId,
            Box<dyn Any + Send + Sync>,
            BuildHasherDefault<TypeIDHasger>,
        >
    >>
);
#[derive(Default)]
struct TypeIDHasger(u64);
impl Hasher for TypeIDHasger {
    #[cold] fn write(&mut self, _: &[u8]) {
        unsafe {std::hint::unreachable_unchecked()}
    }

    #[inline(always)] fn write_u64(&mut self, type_id_value: u64) {
        self.0 = type_id_value
    }
    #[inline(always)] fn finish(&self) -> u64 {
        self.0
    }
}

/// # Memory of a Request
/// 
/// <br>
/// 
/// ```no_run
/// use ohkami::prelude::*;
/// use ohkami::Memory; // <--
/// use std::sync::Arc;
/// 
/// #[tokio::main]
/// async fn main() {
///     let sample_data = Arc::new(String::from("ohkami"));
/// 
///     Ohkami::with(
///         Memory::new(sample_data), // <--
///         (
///             "/hello".GET(hello),
///         )
///     ).howl("0.0.0.0:8080").await
/// }
/// 
/// async fn hello(m: Memory<'_, String>) -> String {
///     /* `*{Memory<'_, T>}` is just `&'_ T` */
///     let name = *m; // <-- &str
/// 
///     format!("Hello, {name}!")
/// }
/// ```
pub struct Memory<Data: Send + Sync + 'static>(Data);
impl<'req, Data: Send + Sync + 'static> super::FromRequest<'req> for Memory<Data> {
    type Error = crate::FromRequestError;

    #[inline]
    fn from_request(req: &'req crate::Request) -> Result<Self, Self::Error> {
        req.memorized::<Data>()
            .map(Memory)
            .ok_or_else(|| {
                #[cfg(debug_assertions)] {
                    eprintln!(
                        "`Memory` of type `{}` was not found",
                        std::any::type_name::<Data>(),
                    );
                }

                crate::FromRequestError::Static("Something went wrong")
            })
    }
}
impl<Data: Send + Sync + 'static> std::ops::Deref for Memory<Data> {
    type Target = Data;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

const _: () = {
    use crate::fang::{Fang, FangProc};

    impl<Data: Clone + Send + Sync + 'static> Memory<Data> {
        pub fn new(data: Data) -> Self {
            Self(data)
        }
    }

    impl<Data: Clone + Send + Sync + 'static, Inner: FangProc>
    Fang<Inner> for Memory<Data> {
        type Proc = UseMemory<Data, Inner>;
        fn chain(self, inner: Inner) -> Self::Proc {
            UseMemory { memory: self, inner_proc: inner }
        }
    }

    pub struct UseMemory<Data: Clone + Send + Sync + 'static, Inner: FangProc> {
        memory:     Memory<Data>,
        inner_proc: Inner,
    }
    impl<Data: Clone + Send + Sync + 'static, Inner: FangProc>
    FangProc for UseMemory<Data, Inner> {
        fn bite<'b>(&'b self, req: &'b mut crate::Request) -> impl std::future::Future<Output = crate::Response> + Send + 'b {
            req.memorize(self.memory.0.clone());
            self.inner_proc.bite(req)
        }
    }
};

#[cfg(test)]
#[test] fn get_easily_the_ref_of_inside_memory_as_satisfying_a_trait() {
    use ::serde_json::Value;

    #[allow(unused)]
    trait T {}
    impl<'t> T for &'t Value {}

    fn _f(_: impl T) {}

    fn _g(m: Memory<Value>) {
        _f(*m)  // <-- easy (just writing `*` before a memory)
    }
}


impl Store {
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub(super) const fn new() -> Self {
        Self(None)
    }

    #[inline] pub fn insert<Data: Send + Sync + 'static>(&mut self, value: Data) {
        self.0.get_or_insert_with(|| Box::new(HashMap::default()))
            .insert(TypeId::of::<Data>(), Box::new(value));
    }

    #[inline] pub fn get<Data: Send + Sync + 'static>(&self) -> Option<&Data> {
        self.0.as_ref()
            .and_then(|map|   map.get(&TypeId::of::<Data>()))
            .and_then(|boxed| boxed.downcast_ref())
    }
}
