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
/// 
/// #[tokio::main]
/// async fn main() {
///     let sample_data = std::Arc::new(String::from("ohkami"));
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
pub struct Memory<'req, Value: Send + Sync + 'static>(&'req Value);
impl<'req, Value: Send + Sync + 'static> super::FromRequest<'req> for Memory<'req, Value> {
    type Error = crate::FromRequestError;
    #[inline] fn from_request(req: &'req crate::Request) -> Result<Self, Self::Error> {
        req.memorized::<Value>()
            .map(Memory)
            .ok_or_else(|| {
                #[cfg(debug_assertions)] {
                    eprintln!(
                        "`Memory` of type `{}` was not found",
                        std::any::type_name::<Value>(),
                    );
                }

                crate::FromRequestError::Static("Something went wrong")
            })
    }
}
impl<'req, Value: Send + Sync + 'static> std::ops::Deref for Memory<'req, Value> {
    type Target = &'req Value;
    #[inline(always)] fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Value: Clone + Send + Sync + 'static> Memory<'_, Value> {
    pub fn new(data: Value) -> impl crate::FrontFang {
        struct Use<Data: Clone + Send + Sync + 'static>(Data);

        impl<Data: Clone + Send + Sync + 'static> crate::FrontFang for Use<Data> {
            type Error = std::convert::Infallible;

            #[inline(always)]
            fn bite(&self, req: &mut crate::Request) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
                req.memorize(self.0.clone());
                async {Ok(())}
            }
        }

        Use(data)
    }
}

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


impl Store {
    #[cfg(any(feature="rt_tokio",feature="rt_async-std"))]
    pub(super) const fn new() -> Self {
        Self(None)
    }

    #[inline] pub fn insert<Value: Send + Sync + 'static>(&mut self, value: Value) {
        self.0.get_or_insert_with(|| Box::new(HashMap::default()))
            .insert(TypeId::of::<Value>(), Box::new(value));
    }

    #[inline] pub fn get<Value: Send + Sync + 'static>(&self) -> Option<&Value> {
        self.0.as_ref()
            .and_then(|map|   map.get(&TypeId::of::<Value>()))
            .and_then(|boxed| boxed.downcast_ref())
    }
}
