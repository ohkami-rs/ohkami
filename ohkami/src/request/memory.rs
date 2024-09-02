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
impl Store {
    #[cfg(feature="__rt__")]
    pub(super) const fn init() -> Self {
        Self(None)
    }

    #[allow(unused)]
    pub fn clear(&mut self) {
        if let Some(map) = &mut self.0 {
            map.clear()
        }
    }

    #[inline] pub fn insert<Data: Send + Sync + 'static>(&mut self, value: Data) {
        self.0.get_or_insert_with(|| Box::new(HashMap::default()))
            .insert(TypeId::of::<Data>(), Box::new(value));
    }

    #[inline] pub fn get<Data: Send + Sync + 'static>(&self) -> Option<&Data> {
        self.0.as_ref().and_then(|map| map
            .get(&TypeId::of::<Data>())
            .map(|boxed| {
                let data: &dyn Any = &**boxed;
                #[cfg(debug_assertions)] {
                    assert!(data.is::<Data>(), "Request's Memory is poisoned!!!");
                }
                unsafe { &*(data as *const dyn Any as *const Data) }
            })
        )
    }
}


/// # Memory of a Request
/// 
/// Memorize and retrieve any data within a request.
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
pub struct Memory<'req, Data: Send + Sync + 'static>(&'req Data);

impl<'req, Data: Send + Sync + 'static>
super::FromRequest<'req> for Memory<'req, Data> {
    type Error = std::convert::Infallible;

    #[inline]
    fn from_request(req: &'req crate::Request) -> Option<Result<Self, Self::Error>> {
        match req.memorized::<Data>().map(Memory) {
            Some(d) => Some(Ok(d)),
            None => {
                #[cfg(debug_assertions)] {
                    crate::warning!(
                        "`Memory` of type `{}` was not found",
                        std::any::type_name::<Data>()
                    )
                }
                None
            }
        }
    }
}
impl<'req, Data: Send + Sync + 'static> std::ops::Deref for Memory<'req, Data> {
    type Target = &'req Data;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

const _: () = {
    use crate::fang::{Fang, FangProc};

    impl<'req, Data: Clone + Send + Sync + 'static> Memory<'req, Data> {
        #[allow(private_interfaces)]
        pub fn new(data: Data) -> UseMemory<Data> {
            UseMemory(data)
        }
    }

    pub struct UseMemory<Data: Clone + Send + Sync + 'static>(
        Data
    );
    impl<Data: Clone + Send + Sync + 'static, Inner: FangProc>
    Fang<Inner> for UseMemory<Data> {
        type Proc = UseMemoryProc<Data, Inner>;
        fn chain(&self, inner: Inner) -> Self::Proc {
            UseMemoryProc { data: self.0.clone(), inner }
        }
    }

    pub struct UseMemoryProc<
        Data:  Clone + Send + Sync + 'static,
        Inner: FangProc,
    > {
        data:  Data,
        inner: Inner,
    }
    impl<Data: Clone + Send + Sync + 'static, Inner: FangProc>
    FangProc for UseMemoryProc<Data, Inner> {
        #[cfg(not(feature="rt_worker"))]
        fn bite<'b>(&'b self, req: &'b mut crate::Request) -> impl std::future::Future<Output = crate::Response> + Send {
            req.memorize(self.data.clone());
            self.inner.bite(req)
        }
        #[cfg(feature="rt_worker")]
        fn bite<'b>(&'b self, req: &'b mut crate::Request) -> impl std::future::Future<Output = crate::Response> {
            req.memorize(self.data.clone());
            self.inner.bite(req)
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

    fn _g(m: Memory<'_, Value>) {
        _f(*m)  // <-- easy (just writing `*` before a memory)
    }
}
