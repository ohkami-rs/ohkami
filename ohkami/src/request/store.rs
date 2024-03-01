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
/// ## memorizing any value
/// With `Request::memorize`：
/// ```
/// use ohkami::{FrontFang, Request, Response};
/// 
/// pub struct MemorizeNow;
/// impl FrontFang for MemorizeNow {
///     type Error = std::convert::Infallible;
///     async fn bite(&self, req: &mut Request) -> Result<(), Self::Error> {
///         req.memorize(serde_json::json!({
///             "now": std::time::SystemTime::now()
///         }));
///         Ok(())
///     }
/// }
/// ```
/// <br>
/// 
/// ## retireiving a reference
/// `*{a Memory<'_, T>}` is just `&'_ T`：
/// ```
/// use ohkami::prelude::*;
/// use ohkami::Memory;  // <---
/// 
/// async fn handler(
///     data: Memory<'_, serde_json::Value>
/// ) -> String {
///         // &'_ Value
///     let memorized_data = *data;
/// 
///     format!(
///         "It's {} !",
///         memorized_data["now"]
///     )
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
    #[cfg(any(feature="rt_tokio",feature="async-std"))]
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
