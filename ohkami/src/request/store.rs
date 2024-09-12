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
